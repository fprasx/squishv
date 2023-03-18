<script lang="ts">
    import { helloworld, parse_riscv } from "./pkg/riscv"
    let input: string = "";
</script>

<main>
    Hello from Svelte!
    { helloworld() }
    <textarea bind:value={input} name="riscv" cols="30" rows="10"></textarea>
    {input}
    {
        JSON.stringify(parse_riscv(input))
    }
</main>

<style>
</style>