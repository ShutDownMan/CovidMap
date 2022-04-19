<script>
	import '../app.css';
	import { page } from '$app/stores';
	import NAVBAR from '$lib/NAVBAR.svelte';
	import NAVITEM from '$lib/NAVITEM.svelte';
	
    let path;

    $: path = $page.url.pathname;
</script>

<NAVBAR logo="/logo.svg" segment={path}>
	<NAVITEM segment={'/'}>Home</NAVITEM>
	<NAVITEM segment={'/about'}>About</NAVITEM>
</NAVBAR>


<slot />
