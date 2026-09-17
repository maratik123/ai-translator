#!/usr/bin/env python3
"""Замер эмбеддинг-моделей на настоящей задаче проекта: RAG по книге.

Задача из `docs/04`: для абзаца найти среди БОЛЕЕ РАННИХ абзацев той же книги те,
что помогут его перевести. Запрос и база — на одном языке (оригинал), обе стороны —
целый абзац. Это симметричное «текст ↔ текст», а не «короткий запрос → документ».

Метрики (эталон строится автоматически, ручной разметки нет):
  RAG EN→EN   золотом считается более ранний абзац, делящий с запросом редкое слово
              книги (df 2-6, без стоп-слов). База ограничена p.id < i, как в SQL из `docs/03`.
  соседство   попадает ли непосредственно предыдущий абзац в top-1/top-3.
  кросс-язык  RU→EN, найти исходник русского абзаца среди всех. Прокси, а не наша задача:
              оставлен для сопоставимости со старыми замерами в `docs/05`.
  разброс     распределение косинусной близости по всем парам абзацев книги. От него
              зависит порог отсечки, и он у каждой модели свой.
  d'          отрыв золотых пар от остальных в единицах разброса остальных.

Значимость считается парным бутстрапом и знаковым тестом против базовой модели:
на 92 абзацах разница в несколько пунктов — шум, и без этого её легко принять за эффект.

ВАЖНО: префикс входа берётся из `config_sentence_transformers.json` модели и для
симметричной задачи ставится на ОБЕ стороны. Без него jina-v5 разваливается
(кросс-язык 54 вместо 98), а EmbeddingGemma теряет 7 пунктов R@3.

Запуск: python3 tools/emb-eval/emb_eval.py [--models bge-m3,jina-v5-small-tm]
Нужны examples/pulp-full.txt и examples/pulp-full-ru.txt (выровнены по абзацам).
"""
import argparse, collections, json, math, pathlib, random, re, signal, statistics, subprocess, time, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
PORT = 8097
OUT = pathlib.Path(__file__).parent / "results.json"

# имя, путь от models/, префикс входа из config_sentence_transformers.json
CONFIGS = [
    ("bge-m3",              "ggml-org/bge-m3-Q8_0-GGUF/bge-m3-q8_0.gguf", ""),
    ("EmbeddingGemma-STS",  "ggml-org/embeddinggemma-300M-GGUF/embeddinggemma-300M-Q8_0.gguf",
                            "task: sentence similarity | query: "),
    ("jina-v5-small-tm",    "jinaai/jina-embeddings-v5-text-small-text-matching-GGUF/v5-small-text-matching-Q8_0.gguf",
                            "Document: "),
    ("jina-v5-nano-tm",     "jinaai/jina-embeddings-v5-text-nano-text-matching-GGUF/v5-nano-text-matching-Q8_0.gguf",
                            "Document: "),
    # проверены и удалены с диска, строки оставлены чтобы замер можно было повторить:
    ("Qwen3-Emb-0.6B",      "Qwen/Qwen3-Embedding-0.6B-GGUF/Qwen3-Embedding-0.6B-Q8_0.gguf", ""),
    ("granite-311m-r2",     "mykor/granite-embedding-311m-multilingual-r2-GGUF/"
                            "granite-embedding-311M-multilingual-r2-Q8_0.gguf", ""),
    ("arctic-embed-l-v2.0", "Casual-Autopsy/snowflake-arctic-embed-l-v2.0-gguf/"
                            "snowflake-arctic-embed-l-v2.0-q8_0.gguf", ""),
    ("harrier-oss-v1-0.6b", "mradermacher/harrier-oss-v1-0.6b-GGUF/harrier-oss-v1-0.6b.Q8_0.gguf", ""),
]
BASELINE = "bge-m3"

STOP = set("""the a an and or but if then than that this these those there here it its it's i me my mine you your
yours he him his she her hers we us our they them their what which who whom whose when where why how all any both
each few more most other some such no nor not only own same so too very can will just don't should now be been
being am is are was were has have had having do does did doing of to in for on with as at by from up down out off
over under again further once about into through during before after above below between don shouldn't couldn't
wouldn't isn't aren't wasn't weren't like get got going went come came back said say says know knew think thought
want wanted make made take took give gave feel felt time way thing things right left good little long""".split())


def paragraphs(path):
    return [x.strip() for x in pathlib.Path(path).read_text().split("\n") if x.strip()]


def build_gold(en):
    """Золотые пары: абзацы, делящие редкое слово книги. Слова наружу не отдаются."""
    words = [{w.lower() for w in re.findall(r"[A-Za-z][A-Za-z'’-]+", p)
              if len(w) >= 4 and w.lower() not in STOP} for p in en]
    df = collections.Counter(w for s in words for w in s)
    rare = [{w for w in s if 2 <= df[w] <= 6} for s in words]
    return {i: {j for j in range(i) if rare[i] & rare[j]} for i in range(len(en))}


def chance(queries, gold, k):
    """Вероятность случайно попасть золотым в top-k — без неё R@k не с чем сравнивать."""
    total = 0.0
    for i in queries:
        base, g, p = i, len(gold[i]), 1.0
        for t in range(k):
            p *= max(0.0, base - g - t) / max(1, base - t)
        total += 1 - p
    return 100 * total / len(queries)


def embed(texts, batch=16):
    out = []
    for i in range(0, len(texts), batch):
        req = urllib.request.Request(
            f"http://127.0.0.1:{PORT}/v1/embeddings",
            data=json.dumps({"input": texts[i:i + batch], "model": "e"}).encode(),
            headers={"Content-Type": "application/json"})
        out += [d["embedding"] for d in json.load(urllib.request.urlopen(req, timeout=900))["data"]]
    return out


def unit(v):
    n = math.sqrt(sum(x * x for x in v)) or 1.0
    return [x / n for x in v]


def cos(a, b):
    return sum(x * y for x, y in zip(a, b))


def ranks_of(emb, queries, gold_of, base_of):
    out = []
    for i in queries:
        order = [j for _, j in sorted(((cos(emb[i], emb[j]), j) for j in base_of(i)), reverse=True)]
        out.append(next((k for k, j in enumerate(order, 1) if j in gold_of(i)), len(order)))
    return out


def summarize(ranks):
    n = len(ranks)
    return (100 * sum(r == 1 for r in ranks) / n,
            100 * sum(r <= 3 for r in ranks) / n,
            100 * sum(1.0 / r for r in ranks) / n)


def serve(model_path, log_path, wait=300):
    """Поднимает llama-server на CPU. --pooling не задаём: он в метаданных GGUF."""
    proc = subprocess.Popen(
        ["llama-server", "-m", str(model_path), "--embedding", "-ub", "2048", "-c", "2048",
         "-np", "1", "-dev", "none", "-ngl", "0", "-t", "8",
         "--host", "127.0.0.1", "--port", str(PORT)],
        stdout=open(log_path, "w"), stderr=subprocess.STDOUT, cwd=ROOT)
    t0 = time.time()
    while time.time() - t0 < wait:
        if proc.poll() is not None:
            return proc, False
        if "listening on" in pathlib.Path(log_path).read_text():
            return proc, True
        time.sleep(1)
    return proc, False


def stop(proc):
    proc.send_signal(signal.SIGTERM)
    try:
        proc.wait(timeout=60)
    except Exception:
        proc.kill()
    time.sleep(2)


def measure(name, rel, prefix, en_txt, ru_txt, gold, queries, adjacent, logdir):
    model = ROOT / "models" / rel
    if not model.exists():
        print(f"{name:22} пропуск: нет файла {rel}", flush=True)
        return None
    proc, ok = serve(model, logdir / f"{name}.log")
    try:
        if not ok:
            print(f"{name:22} НЕ ЗАПУСТИЛСЯ, смотри {logdir/f'{name}.log'}", flush=True)
            return None
        t0 = time.time()
        en = [unit(v) for v in embed([prefix + x for x in en_txt])]
        ms = (time.time() - t0) / len(en_txt) * 1000
        ru = [unit(v) for v in embed([prefix + x for x in ru_txt])]

        rag = ranks_of(en, queries, lambda i: gold[i], lambda i: range(i))
        adj = ranks_of(en, adjacent, lambda i: {i - 1}, lambda i: range(i))
        cross = []
        for i, q in enumerate(ru):
            order = [j for _, j in sorted(((cos(q, en[j]), j) for j in range(len(en))), reverse=True)]
            cross.append(order.index(i) + 1)

        gold_sims, other_sims = [], []
        for i in range(len(en)):
            for j in range(i):
                (gold_sims if j in gold[i] else other_sims).append(cos(en[i], en[j]))
        allp = sorted(gold_sims + other_sims)
        sd = statistics.pstdev(other_sims) or 1e-9
        row = dict(name=name, prefix=prefix, dim=len(en[0]), ms=ms,
                   rag=summarize(rag), adj=summarize(adj), cross=summarize(cross),
                   rag_ranks=rag, adj_ranks=adj, cross_ranks=cross,
                   median=statistics.median(allp), p95=allp[int(0.95 * len(allp))], max=allp[-1],
                   d=(statistics.mean(gold_sims) - statistics.mean(other_sims)) / sd)
        print(f"{name:22} dim {row['dim']:4} {ms:4.0f} мс | RAG R@1 {row['rag'][0]:5.1f} R@3 {row['rag'][1]:5.1f}"
              f" MRR {row['rag'][2]:5.1f} | сосед R@3 {row['adj'][1]:5.1f} | кросс R@1 {row['cross'][0]:5.1f}"
              f" | медиана {row['median']:.3f} p95 {row['p95']:.3f} | d' {row['d']:4.2f}", flush=True)
        return row
    finally:
        stop(proc)


def significance(rows, baseline, resamples=20000, seed=11):
    base = next((r for r in rows if r["name"] == baseline), None)
    if base is None or len(rows) < 2:
        return
    random.seed(seed)
    n = len(base["rag_ranks"])
    draws = [[random.randrange(n) for _ in range(n)] for _ in range(resamples)]

    def stat(ranks, sel, kind):
        if kind == "R@1":
            return 100 * sum(ranks[i] == 1 for i in sel) / len(sel)
        if kind == "R@3":
            return 100 * sum(ranks[i] <= 3 for i in sel) / len(sel)
        return 100 * sum(1.0 / ranks[i] for i in sel) / len(sel)

    others = [r for r in rows if r is not base]
    print(f"\nRAG-задача против {baseline}: парный бутстрап ({resamples} ресэмплов, n={n}) и знаковый тест.")
    print(f"{'модель':<22}{'ΔR@1':>20}{'ΔR@3':>20}{'ΔMRR':>20}{'знак. p':>10}{'поправка':>10}")
    for r in others:
        cells = []
        for kind in ("R@1", "R@3", "MRR"):
            d = sorted(stat(r["rag_ranks"], s, kind) - stat(base["rag_ranks"], s, kind) for s in draws)
            lo, hi = d[int(0.025 * resamples)], d[int(0.975 * resamples)]
            point = stat(r["rag_ranks"], range(n), kind) - stat(base["rag_ranks"], range(n), kind)
            cells.append(f"{point:+5.1f} [{lo:+5.1f},{hi:+5.1f}]" + ("*" if lo > 0 or hi < 0 else " "))
        better = sum(x < y for x, y in zip(r["rag_ranks"], base["rag_ranks"]))
        worse = sum(x > y for x, y in zip(r["rag_ranks"], base["rag_ranks"]))
        m = better + worse
        p = min(1.0, 2 * sum(math.comb(m, i) for i in range(min(better, worse) + 1)) / 2 ** m) if m else 1.0
        print(f"{r['name']:<22}" + "".join(f"{c:>20}" for c in cells)
              + f"{p:>10.4f}{min(1.0, p*len(others)):>10.3f}")
    print("* — доверительный интервал не накрывает ноль; «поправка» — p, умноженное на число сравнений.")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--models", help="через запятую; по умолчанию все, что есть на диске")
    ap.add_argument("--en", default=ROOT / "examples/pulp-full.txt")
    ap.add_argument("--ru", default=ROOT / "examples/pulp-full-ru.txt")
    ap.add_argument("--logs", default=pathlib.Path("/tmp") / "emb-eval-logs")
    args = ap.parse_args()

    en_txt, ru_txt = paragraphs(args.en), paragraphs(args.ru)
    if len(en_txt) != len(ru_txt):
        raise SystemExit(f"не выровнены: {len(en_txt)} абзацев оригинала против {len(ru_txt)} перевода")
    logdir = pathlib.Path(args.logs)
    logdir.mkdir(parents=True, exist_ok=True)

    gold = build_gold(en_txt)
    queries = [i for i in gold if gold[i] and i >= 10]
    adjacent = list(range(11, len(en_txt)))
    print(f"книга: {len(en_txt)} абзацев | RAG-запросов {len(queries)}, случайный уровень "
          f"R@1 {chance(queries, gold, 1):.1f}% R@3 {chance(queries, gold, 3):.1f}%"
          f" | соседей {len(adjacent)}, случайный R@3 {chance(adjacent, {i: {i-1} for i in adjacent}, 3):.1f}%\n")

    wanted = args.models.split(",") if args.models else None
    rows = [r for r in (measure(n, rel, pre, en_txt, ru_txt, gold, queries, adjacent, logdir)
                        for n, rel, pre in CONFIGS if not wanted or n in wanted) if r]
    significance(rows, BASELINE)
    OUT.write_text(json.dumps(rows, ensure_ascii=False, indent=1))
    print(f"\nсырые ранги и числа: {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
