<!--
(c) Matthew Boyer, 2023.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

This Source Code Form is "Incompatible With Secondary Licenses", as
defined by the Mozilla Public License, v. 2.0.
-->

<script lang="ts">
    import {invoke} from "@tauri-apps/api/tauri";
    import {WebviewWindow} from "@tauri-apps/api/window";
    import {open} from "@tauri-apps/api/dialog";

    let decks

    let add_deck = false
    let new_deck_name: ""

    const load = async () => { decks = await invoke("get_decks") }

    const newWindow = async (deck: string) => {
        new WebviewWindow("deck", {url: "/deck/" + deck})
            .once("tauri://error", (e) => {console.error(e)})
            .then()
    }

    const new_deck = async () => {await invoke("new_deck", {deck: new_deck_name})}
    const import_deck = async () => {await invoke("import_deck", {path: await open()})}

    load()
</script>

<div>
    <a href="/"><button>Back</button></a>
    <button on:click={() => {add_deck = true}}>New</button>
    <button on:click={import_deck}>Import</button>
    {#if decks !== undefined}
        {#each decks as deck}
            <br /><button on:click={() => newWindow(deck)}>{deck}</button>
        {/each}
        {#if add_deck === true}
            <input type="text" bind:value={new_deck_name} /><button on:click={new_deck}>Ok</button><button on:click={() => {
                add_deck = false
                new_deck_name = ""
            }}>Cancel</button>
        {/if}
    {/if}
</div>
