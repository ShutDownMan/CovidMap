<script>
	import { getContext } from 'svelte';

	export let segment = null;
	const current = getContext('nav');

	function setSegment() {
		current.set(segment);
	}
</script>

<li on:click={setSegment}>
	<a
		href={segment}
        class={($current === segment) ? "navbar-a-active" : "navbar-a"}
	>
		<slot />
	</a>
</li>

<!-- 
<li>
    <a href="/" class="navbar-a" aria-current="page">Home</a>
</li>
<li>
    <a href="/about" class="navbar-a-active">About</a>
</li> -->
