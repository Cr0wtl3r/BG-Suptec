<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AtivarOffice } from "../../../wailsjs/go/main/App";

  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let versaoSelecionada = "2024";

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando ativação..."];
    try {
      await AtivarOffice(versaoSelecionada);
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
      emExecucao = false;
    }
  }

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    logLines = [...logLines, `[${timestamp}] ${mensagem}`];
    if (mensagem.includes("---")) {
      emExecucao = false;
    }
  }

  onMount(() => {
    const eventName = "log:ativacao:office";
    EventsOn(eventName, adicionarLog);
    return () => {
      EventsOff(eventName);
    };
  });
</script>

<FeatureRunner
  titulo="Ativação do Office"
  descricao="Esta função fechará todos os aplicativos do Office (Word, Excel, etc.) e realizará a ativação utilizando servidores KMS públicos. Requer privilégios de Administrador."
  textoBotao="Iniciar Ativação do Office"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
>
  <div class="mb-6">
    <label for="versao-office" class="font-bold text-text-light mb-2 block"
      >Selecione a versão do Office:</label
    >
    <select
      id="versao-office"
      bind:value={versaoSelecionada}
      disabled={emExecucao}
      class="w-full p-3 bg-dark-blue-light text-text-light border border-primary-purple rounded-md text-base focus:outline-none focus:ring-1 focus:ring-primary-purple disabled:opacity-60 disabled:cursor-not-allowed"
    >
      <option value="2024">Office 2024</option>
      <option value="2021">Office 2021</option>
      <option value="2016">Office 2016</option>
    </select>
  </div>
</FeatureRunner>
