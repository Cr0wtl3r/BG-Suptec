<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { EventsOn, EventsOff } from "../../../wailsjs/runtime";
  import { AtivarWindows } from "../../../wailsjs/go/main/App";
  import FeatureRunner from "./FeatureRunner.svelte";

  const dispatch = createEventDispatcher();

  let logLines: string[] = ["Aguardando início..."];
  let emExecucao = false;
  let versaoSelecionada = "pro";

  async function iniciar() {
    emExecucao = true;
    logLines = ["Iniciando ativação do Windows..."];
    try {
      await AtivarWindows(versaoSelecionada);
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
      emExecucao = false;
    }
  }

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    logLines = [...logLines, `[${timestamp}] ${mensagem}`];

    const mensagemFinal =
      mensagem.includes("--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---") ||
      mensagem.includes("--- FALHA NA ATIVAÇÃO ---");

    if (mensagemFinal) {
      emExecucao = false;
    }
  }

  function onWindowsProcessoFinalizado() {
    emExecucao = false;
  }

  onMount(() => {
    const eventName = "log:ativacao:windows";
    const finalizadoEventName = "ativacao:windows:finalizado";

    EventsOn(eventName, adicionarLog);
    EventsOn(finalizadoEventName, onWindowsProcessoFinalizado);

    return () => {
      EventsOff(eventName);
      EventsOff(finalizadoEventName);
    };
  });
</script>

<FeatureRunner
  titulo="Ativação do Windows"
  descricao="Esta função tentará ativar sua versão do Windows usando servidores KMS públicos. Requer privilégios de Administrador."
  textoBotao="Iniciar Ativação do Windows"
  bind:logLines
  bind:emExecucao
  on:start={iniciar}
  on:voltar={() => dispatch("voltar")}
>
  <div class="mb-6">
    <label for="versao-windows" class="font-bold text-text-light mb-2 block"
      >Selecione a versão do Windows:</label
    >
    <select
      id="versao-windows"
      bind:value={versaoSelecionada}
      disabled={emExecucao}
      class="w-full p-3 bg-dark-blue-light text-text-light border border-primary-purple rounded-md text-base focus:outline-none focus:ring-1 focus:ring-primary-purple disabled:opacity-60 disabled:cursor-not-allowed"
    >
      <option value="pro">Windows 10/11 Pro</option>
      <option value="home">Windows 10/11 Home</option>
      <option value="education">Windows 10/11 Education</option>
      <option value="enterprise">Windows 10/11 Enterprise</option>
    </select>
  </div>
</FeatureRunner>
