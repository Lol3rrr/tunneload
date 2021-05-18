<script lang="ts">
	export let header: Array<String> = [];
	export let content: Array<Array<String>> = [];
	export let row_click: (row: Number) => () => void = (row) => {
		return () => {
			console.log(row);
		};
	};
</script>

<table>
	<tr>
		{#each header as tmp}
			<th>{tmp}</th>
		{/each}
	</tr>
	{#each content as line, i}
		<tr on:click="{row_click(i)}">
			{#each line as part}
				<td>{part}</td>
			{/each}
		</tr>
	{/each}
</table>

<style>
	table {
		margin: auto;
	}
	table, th, td {
		border: none;
		border-collapse: collapse;
	}
	th, td {
		font-size: 22px;
		color: var(--white);
	}
	td {
		text-align: left;
		padding: 5px 10px 5px 10px;
	}
	td:nth-child(2) {
		text-align: right;
	}

	tr {
		background-color: #525557;
		transition: 0.2s;
	}
	tr:nth-child(2n) {
		background-color: var(--grey);
	}
	tr:not(tr:nth-child(1)):hover {
		transform: scale(1.05, 1.1);
	}
</style>
