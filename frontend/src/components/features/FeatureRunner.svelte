<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import FeatureContainer from "../shared/FeatureContainer.svelte";
  import LogPanel from "../shared/LogPanel.svelte";
  import BotaoVoltar from "../shared/BotaoVoltar.svelte";

  export let titulo: string;
  export let descricao: string;
  export let textoBotao: string = "Executar";
  export let logLines: string[] = [];
  export let emExecucao: boolean = false;

  const dispatch = createEventDispatcher();
</script>

<FeatureContainer {titulo}>
  <div class="flex-grow flex flex-col min-h-0 overflow-y-auto pr-2">
    <p class="mt-0 mb-6 opacity-90">{descricao}</p>

    <slot></slot>

    <button
      class="w-full p-4 text-xl font-bold cursor-pointer bg-accent-orange text-dark-blue-bg border-none rounded-lg transition-all duration-200 hover:brightness-110 disabled:bg-gray-600 disabled:cursor-not-allowed"
      on:click={() => dispatch("start")}
      disabled={emExecucao}
    >
      {emExecucao ? "Executando..." : textoBotao}
    </button>

    <LogPanel {logLines} />
  </div>

  <div class="flex-shrink-0 pt-4">
    <BotaoVoltar on:click={() => dispatch("voltar")} />
  </div>
</FeatureContainer>
