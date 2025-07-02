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
  <p>{descricao}</p>

  <slot></slot>

  <button
    class="btn-executar"
    on:click={() => dispatch("start")}
    disabled={emExecucao}
  >
    {emExecucao ? "Executando..." : textoBotao}
  </button>

  <LogPanel {logLines} />

  <BotaoVoltar on:click={() => dispatch("voltar")} />
</FeatureContainer>

<style>
  p {
    margin-top: 0;
    margin-bottom: 25px;
    opacity: 0.9;
  }
</style>
