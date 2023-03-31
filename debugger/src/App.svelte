<script lang="ts">
	import CodeDisplay from './CodeDisplay.svelte';
	import { parse_instruction, parse_riscv } from './pkg/riscv';
	import { source } from './stores/source';
</script>

<main>
	<p>Enter your code here:</p>
	<textarea bind:value={$source} name="riscv" cols="30" rows="10" />
	{JSON.stringify(parse_riscv($source))}
    {#each $source.split('\n') as line}
        <div>{JSON.stringify(parse_instruction(line))}</div> 
    {/each}
	<CodeDisplay />
</main>

<style>
</style>
