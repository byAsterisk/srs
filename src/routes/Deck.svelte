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
    import MarkdownIt from 'markdown-it'
    import * as ruby from 'markdown-it-ruby'
    import {WebviewWindow} from "@tauri-apps/api/window";
    import {save} from "@tauri-apps/api/dialog";

    const md = new MarkdownIt()
    md.use(ruby)

    export let deck
    let cards
    let activeCard
    let rename = false
    let new_deck_name: ""

    const load = async () => {
        let cards_raw = await invoke("get_deck", { deck: deck })
        cards = {}
        for (let card of cards_raw) {cards[card[1]] = [card[2], card[3]]}
        console.log(cards)
    }

    const new_card = async () => {
        let row = await invoke("new_card", {deck})
        await load()
        activeCard = row
    }
    const edit_card = async () => {
        await invoke("edit_card", {deck: deck, id: activeCard, obverse: cards[activeCard][0], reverse: cards[activeCard][1]})
    }
    const reset = async () => {await invoke("reset_card", {deck: deck, id: activeCard})}
    const delete_card = async () => {await invoke("delete_card", {deck: deck, id: Number(activeCard)})}

    const export_deck = async () => {await invoke("export_deck", {deck: deck, path: await save()})}

    const rename_deck = async () => {
        await invoke("rename_deck", {deck: deck, name: new_deck_name})
        location.href = "/deck/" + new_deck_name
    }

    const delete_deck = async () => {
        let window = await WebviewWindow.getFocusedWindow()
        await invoke("delete_deck", {deck: deck})
        window?.close()
    }

    load()
</script>

<div>
    <div>
        <p>{deck}</p>
        <button on:click={export_deck}>Export</button>
        <button on:click={() => {rename = true}}>Rename</button>
        <button on:click={delete_deck}>Delete</button>
        {#if rename === true}
            <input type="text" bind:value={new_deck_name} /><button on:click={rename_deck}>Ok</button><button on:click={() => {
                rename = false
                new_deck_name = ""
            }}>Cancel</button>
        {/if}
        <br />
        <button on:click={new_card}>New</button>
        {#if cards !== undefined}
            {#each Object.entries(cards) as [index, card]}
                <div on:click={() => {activeCard = index}}>{@html md.render(card[0])}</div><br />
            {/each}
        {/if}
    </div>
    <div>
        {#if activeCard !== undefined}
            <input type="text" bind:value={cards[activeCard][0]} /><br />
            <input type="text" bind:value={cards[activeCard][1]} /><br />
            <button on:click={edit_card}>Save</button>
            <button on:click={reset}>Reset</button>
            <button on:click={delete_card}>Delete</button>
        {:else}
            <p>No active card.</p>
        {/if}
    </div>
</div>