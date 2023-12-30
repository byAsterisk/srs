<!--
(c) Matthew Boyer, 2023.

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.

This Source Code Form is "Incompatible With Secondary Licenses", as
defined by the Mozilla Public License, v. 2.0.
-->

<script lang="ts">
    import { Router, Route } from "svelte-routing";
    import Index from "./routes/Index.svelte";
    import Review from "./routes/Review.svelte";
    import Settings from "./routes/Settings.svelte";
    import Decks from "./routes/Decks.svelte";
    import Deck from "./routes/Deck.svelte";

    let url = ""
</script>

<Router {url}>
    <Route path="/"><Index /></Route>
    <Route path="/review"><Review /></Route>
    <Route path="/settings"><Settings /></Route>
    <Route path="/decks"><Decks /></Route>
    <Route path="/deck/*deck" let:params><Deck deck={params["deck"]}/></Route>
</Router>