<!--
(c) Matthew Boyer, 2023.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

This Source Code Form is "Incompatible With Secondary Licenses", as
defined by the Mozilla Public License, v. 2.0.
-->

<script lang="ts">
    import {invoke} from '@tauri-apps/api/tauri'
    import {unified} from "unified";
    import remarkParse from "remark-parse";
    import remarkRehype from "remark-rehype";
    import rehypeStringify from "rehype-stringify";
    import remarkRuby from "remark-denden-ruby";


    let card: [String, String]
    let show = false
    const md = unified()
        .use(remarkParse)
        .use(remarkRuby)
        .use(remarkRehype)
        .use(rehypeStringify)

    const next_card = async () => {
        show = false
        card = await invoke("next_card")
        console.log(card)
    }

    const flip = async () => show = true;

    const again = async () => {
        await invoke("update_card", { rating: 1 })
        await next_card()
    }

    const hard = async () => {
        await invoke("update_card", { rating: 2 })
        await next_card()
    }

    const good = async () => {
        await invoke("update_card", { rating: 3 })
        await next_card()
    }

    const easy = async () => {
        await invoke("update_card", { rating: 4 })
        await next_card()
    }

    next_card()
</script>

<a href="/"><button>Back</button></a>
<div>
    {#if card !== undefined}
        {#if card.length !== 0}
            {@html md.processSync(card[0])}<br/>
            {#if show}
                <hr/>{@html md.processSync(card[1])}<br/>
                <button on:click={again}>Again</button>
                <button on:click={hard}>Hard</button>
                <button on:click={good}>Good</button>
                <button on:click={easy}>Easy</button>
            {:else}
                <button on:click={flip}>Flip</button>
            {/if}
        {:else}
            <p>Done</p>
        {/if}
    {/if}
</div>