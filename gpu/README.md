# GPU experiments

`probe_local_models.ipynb` measures whether local models can file claims hc accepts: decode-time schema enforcement (Ollama `format`) against prompt-only, scored as valid on first try and correct.

On a local GPU (for example an RTX 3070 Ti) the same run is:

```sh
ollama serve &
ollama pull qwen3:8b
cargo build --release -p hc
./target/release/hc probe-model --model qwen3:8b --repeats 3 > probe.json
```

What the result decides: a model that is reliably valid and correct with the schema enforced can report claims directly (for example a local agent such as Neon); one that is not should have its claims formatted by a second, constrained call. It is also the cheapest place to test the research finding that decode-time constraints do not cost accuracy when the reasoning field comes before the verdict.
