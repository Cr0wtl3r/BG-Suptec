<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();
</script>

<button
  class="w-fit mt-6 p-2.5 px-5 border border-primary-purple bg-transparent text-text-light rounded-lg cursor-pointer text-base font-bold transition-all duration-200 hover:bg-primary-purple block mx-auto"
  on:click={() => dispatch("click")}
>
  &larr; Voltar ao Painel Principal
</button>
