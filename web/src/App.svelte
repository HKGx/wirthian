<script lang="ts">
    import Editor from "./lib/Editor.svelte";
    import { run_source } from "./lib/wasm";
    import type { Diagnostic } from "@codemirror/lint";

    const examples = Object.entries(
        import.meta.glob("../../programs/*.pa", {
            query: "?raw",
            import: "default",
            eager: true,
        }),
    )
        .map(([path, code]) => ({
            name: path.split("/").pop()!.replace(/\.pa$/, ""),
            code: code as string,
        }))
        .sort((a, b) => a.name.localeCompare(b.name));

    let selected = $state(examples[0]?.name ?? "");
    let source = $state(examples[0]?.code ?? "");
    let stdin = $state("");
    let output = $state("");
    let error = $state<string | null>(null);
    let diagnostics = $state<Diagnostic[]>([]);
    let running = $state(false);

    function pick(name: string) {
        const ex = examples.find((e) => e.name === name);
        if (ex) {
            selected = name;
            source = ex.code;
            output = "";
            error = null;
            diagnostics = [];
        }
    }

    function run() {
        running = true;
        try {
            const r = run_source(source, stdin);
            output = r.output;
            error = r.error ?? null;
            diagnostics = r.diagnostics as Diagnostic[];
        } catch (e) {
            error = String(e);
            output = "";
            diagnostics = [];
        } finally {
            running = false;
        }
    }
</script>

<main>
    <h1>Playground</h1>
    <section class="grid">
        <div class="editor-pane">
            <div class="editor-head">
                <label for="editor">Kod źródłowy</label>
                <select
                    onchange={(e) => pick(e.currentTarget.value)}
                    value={selected}
                >
                    {#each examples as ex (ex.name)}
                        <option value={ex.name}>{ex.name}</option>
                    {/each}
                </select>
            </div>
            <div class="editor-wrap">
                <Editor
                    bind:value={source}
                    {diagnostics}
                    oninput={() => (diagnostics = [])}
                />
            </div>
        </div>
        <div class="side-pane">
            <label for="stdin">Wejście (stdin)</label>
            <textarea id="stdin" bind:value={stdin} rows="6"></textarea>
            <button onclick={run} disabled={running}>
                {running ? "Wykonywanie…" : "Uruchom"}
            </button>
            <label for="output">{error ? "Błąd wykonania" : "Wyjście"}</label>
            <pre id="output" class={error ? "err" : ""}>{error ?? output}</pre>
        </div>
    </section>
</main>

<style>
    main {
        max-width: 1200px;
        margin: 0 auto;
        padding: 1rem;
    }
    h1 {
        font-size: 1.25rem;
        margin: 0 0 1rem;
    }
    .grid {
        display: grid;
        grid-template-columns: 1.4fr 1fr;
        gap: 1rem;
        height: calc(100vh - 6rem);
    }
    label {
        display: block;
        font-size: 0.85rem;
        margin-bottom: 0.25rem;
        color: #555;
    }
    .editor-pane {
        display: flex;
        flex-direction: column;
    }
    .editor-head {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 0.25rem;
    }
    .editor-head label {
        margin-bottom: 0;
    }
    select {
        font-size: 0.85rem;
        padding: 0.15rem 0.3rem;
    }
    .editor-wrap {
        flex: 1;
        border: 1px solid #ccc;
        overflow: hidden;
    }
    .side-pane {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    textarea {
        font-family: monospace;
        font-size: 13px;
        resize: vertical;
    }
    button {
        align-self: flex-start;
        padding: 0.5rem 1.5rem;
        font-size: 1rem;
        cursor: pointer;
    }
    button:disabled {
        cursor: wait;
        opacity: 0.6;
    }
    #output {
        flex: 1;
        margin: 0;
        padding: 0.5rem;
        background: #f7f7f7;
        border: 1px solid #ccc;
        overflow: auto;
        font-family: monospace;
        font-size: 13px;
        white-space: pre-wrap;
    }
    #output.err {
        color: #c00;
        background: #fee;
        border-color: #c00;
    }
</style>
