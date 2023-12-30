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

    let loaded = false
    let cardCount: number

    const load = async () => {
        cardCount = await invoke("card_count");
        loaded = true
    }

    load()
</script>

{#if loaded}
    <a href="/review"><button>Review ({cardCount})</button></a>
    <a href="/settings"><button>Settings</button></a>
    <a href="/decks"><button>Decks</button></a>
{/if}
