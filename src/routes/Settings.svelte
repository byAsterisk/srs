<!--
(c) Matthew Boyer, 2023.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

This Source Code Form is "Incompatible With Secondary Licenses", as
defined by the Mozilla Public License, v. 2.0.
-->

<script lang="ts">
    import {invoke} from "@tauri-apps/api/tauri"

    let fetched = false
    let new_cards: number

    invoke("get_settings").then((result) => {
        new_cards = result["new_cards"]
        fetched = true
    })

    const save = async () => {
        await invoke("set_settings", {value: {
            new_cards: new_cards
        }})
        location.href = "/"
    }
</script>

<div>
    <button on:click={save}>Back</button>
    {#if fetched}
        <input
            type="number"
            bind:value={new_cards}
        />
    {/if}
    <p>
        (c) Matthew Boyer, 2023.
        This project is licensed under the Mozilla Public License (mozilla.org/MPL/2.0), marked "Incompatible with Secondary Licenses".
        Source code: github.com/byAsterisk/srs
    </p>
</div>