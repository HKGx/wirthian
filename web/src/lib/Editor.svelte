<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { EditorState } from "@codemirror/state";
    import { EditorView } from "@codemirror/view";
    import { basicSetup } from "codemirror";
    import { wirthian } from "../lang/wirthian";

    let {
        value = $bindable(""),
        oninput,
    }: {
        value: string;
        oninput?: (v: string) => void;
    } = $props();

    let host: HTMLDivElement;
    let view: EditorView | null = null;

    onMount(() => {
        view = new EditorView({
            state: EditorState.create({
                doc: value,
                extensions: [
                    basicSetup,
                    wirthian(),
                    EditorView.theme({
                        "&": { height: "100%", fontSize: "13px" },
                        ".cm-scroller": { fontFamily: "monospace" },
                    }),
                    EditorView.updateListener.of((u) => {
                        if (u.docChanged) {
                            value = u.state.doc.toString();
                            oninput?.(value);
                        }
                    }),
                ],
            }),
            parent: host,
        });
    });

    onDestroy(() => view?.destroy());

    $effect(() => {
        if (view && value !== view.state.doc.toString()) {
            view.dispatch({
                changes: { from: 0, to: view.state.doc.length, insert: value },
            });
        }
    });
</script>

<div bind:this={host} class="cm-host"></div>

<style>
    .cm-host {
        height: 100%;
        overflow: auto;
    }
    .cm-host :global(.cm-editor) {
        height: 100%;
    }
</style>
