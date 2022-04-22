<script lang="ts">
	import DOCUMENT_MODAL from '$lib/DOCUMENT_MODAL.svelte';

	let showModal = false;
	let modalDoc = undefined;
	const showModalDoc = (doc) => {
		showModal = true;
		modalDoc = doc;
	};

	let defaultSearches = [
		'what are the effects of coronavirus or covid on pregnant women?',
		'When is the salivary viral load highest for COVID-19?',
		'what are the coronavirus side effects and tribulations?',
		'what are the long term effects of corona virus disease Sars-Cov-2?',
		'how can the coronavirus mutations occour?',
		'which socioeconomical impacts does the coronavírus have on underdeveloped countries?',
		'what are the effective medication and safety approaches to coronavírus disease?',
		'What is the political landscape of the coronavirus pandemic?',
		'what is the aftermath of the pandemic?',
		'Is convalescent plasma therapy a precursor to vaccine?',
		''
	];

	let searchDocs = {
		searchQuery: defaultSearches[Math.floor(Math.random() * defaultSearches.length)],
		limit: 20,
		docs: []
	};

	async function clickSearch(e: Event) {
		// console.log('Searching');
		// console.log(e);

		let fetchedDocs = await fetch(`http://192.168.1.220:3000/search/context`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				search_query: searchDocs.searchQuery,
				limit: 10
			})
		}).then((response) => response.json());

		// console.log(fetchedDocs);

		searchDocs.docs = fetchedDocs.search_results;
	}

	function getTruncatedText(text: string, maxLength: number): string {
		if (text.length <= maxLength) {
			return text;
		}
		return `${text.slice(0, maxLength)} [...]`;
	}
</script>

<div class="body flex justify-center">
	<div
		class="container w-4/5 max-w-4xl justify-items-center px-8 py-4 mt-4 grid grid-rows-2 bg-gray-800 rounded-xl"
		style="grid-template-rows: auto 1fr;"
	>
		<div class="container flex justify-between gap-4">
			<input
				type="text"
				name="search"
				id="input-search"
				placeholder="Ask something here"
				on:keypress={(e) => {
					if (e.key === 'Enter') clickSearch(e);
				}}
				bind:value={searchDocs.searchQuery}
				class="px-4 py-2 w-full border-2 border-gray-500 hover:border-transparent rounded-xl shadow-md flex-1 transition-all duration-300"
			/>
			<button
				on:click={clickSearch}
				class="bg-cyan-400 hover:bg-cyan-500 rounded-xl shadow-md px-4 my-1 flex-grow-0 font-mono font-bold text-gray-900 border-2 border-transparent hover:border-white"
				>Search</button
			>
		</div>

		<div class="py-4 flex flex-col gap-8">
			{#each searchDocs.docs as doc}
				<a href="#" on:click={() => showModalDoc(doc)}>
					<div class="container py-4 bg-black bg-opacity-25 hover:bg-opacity-40 rounded-2xl">
						<h2 class="px-4 font-bold font-mono text-cyan-400 text-lg underline">
							{doc.title}
						</h2>

						<div class="px-8">
							<p class=" font-mono text-white text-justify">
								{getTruncatedText(doc.abstract_text, 250)}
							</p>
						</div>
					</div>
				</a>
			{/each}
		</div>
	</div>
</div>

<!-- FULL ABSTRACT MODAL -->
{#if modalDoc}
	<DOCUMENT_MODAL
		title={modalDoc.title}
		open={showModal}
		on:close={() => {
			showModal = false;
		}}
	>
		<svelte:fragment slot="body"
			><p class="font-mono text-sm">{modalDoc.abstract_text}</p></svelte:fragment
		>
	</DOCUMENT_MODAL>
{/if}

<style>
	div.body {
		/* @apply ; */
	}
</style>
