<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    ObterProgramasInstalados,
    ListarExecutaveis,
    VerificarStatusFirewall,
    BloquearProgramasFirewall,
    DesbloquearProgramasFirewall,
    SelecionarArquivoExe,
  } from "../../../wailsjs/go/main/App";
  import FeatureContainer from "../shared/FeatureContainer.svelte";
  import BotaoVoltar from "../shared/BotaoVoltar.svelte";

  type ProgramaInfo = {
    nome: string;
    caminho: string;
  };

  type ExecutavelInfo = {
    caminho: string;
    bloqueado: boolean;
  };

  const dispatch = createEventDispatcher();

  let programasInstalados: ProgramaInfo[] = [];
  let executaveisParaAcao: ExecutavelInfo[] = [];
  let processando = false;
  let erro = "";
  let mensagemSucesso = "";

  onMount(async () => {
    processando = true;
    try {
      programasInstalados = await ObterProgramasInstalados();
    } catch (e) {
      erro = `Erro ao carregar programas instalados: ${e}`;
    } finally {
      processando = false;
    }
  });

  async function adicionarExecutaveis(novosCaminhos: string[]) {
    processando = true;
    erro = "";
    mensagemSucesso = "";
    try {
      const status = await VerificarStatusFirewall(novosCaminhos);
      for (const caminho of novosCaminhos) {
        if (!executaveisParaAcao.some((e) => e.caminho === caminho)) {
          executaveisParaAcao.push({ caminho, bloqueado: status[caminho] });
        }
      }
      executaveisParaAcao = [...executaveisParaAcao];
    } catch (e) {
      erro = `Erro ao verificar status no firewall: ${e}`;
    } finally {
      processando = false;
    }
  }

  async function selecionarArquivo() {
    try {
      const caminho = await SelecionarArquivoExe();
      if (caminho) {
        await adicionarExecutaveis([caminho]);
      }
    } catch (e) {
      if (e !== "user cancelled") {
        erro = `Erro ao selecionar arquivo: ${e}`;
      }
    }
  }

  async function selecionarProgramaInstalado(event: Event) {
    const target = event.target as HTMLSelectElement;
    const caminhoPasta = target.value;
    if (!caminhoPasta) return;

    processando = true;
    try {
      const executaveis = await ListarExecutaveis(caminhoPasta);
      if (executaveis.length > 0) {
        await adicionarExecutaveis(executaveis);
      } else {
        erro = "Nenhum arquivo .exe encontrado na pasta deste programa.";
      }
    } catch (e) {
      erro = `Erro ao listar executáveis: ${e}`;
    } finally {
      processando = false;
      target.value = "";
    }
  }

  function removerExecutavel(caminho: string) {
    executaveisParaAcao = executaveisParaAcao.filter(
      (e) => e.caminho !== caminho,
    );
  }

  async function executarAcao(acao: "bloquear" | "desbloquear") {
    const caminhos = executaveisParaAcao.map((e) => e.caminho);
    if (caminhos.length === 0) {
      erro = "Nenhum programa na lista para aplicar a ação.";
      return;
    }

    processando = true;
    erro = "";
    mensagemSucesso = "";
    try {
      if (acao === "bloquear") {
        await BloquearProgramasFirewall(caminhos);
        mensagemSucesso = "Programas bloqueados com sucesso!";
      } else {
        await DesbloquearProgramasFirewall(caminhos);
        mensagemSucesso = "Programas desbloqueados com sucesso!";
      }
      const statusAtualizado = await VerificarStatusFirewall(caminhos);
      executaveisParaAcao.forEach((exe) => {
        exe.bloqueado = statusAtualizado[exe.caminho];
      });
      executaveisParaAcao = [...executaveisParaAcao];
    } catch (e) {
      erro = `Erro ao ${acao} programas: ${e}`;
    } finally {
      processando = false;
    }
  }
</script>

<FeatureContainer titulo="Bloqueador de Programas no Firewall">
  <div class="flex-grow flex flex-col min-h-0">
    <div class="flex-shrink-0 space-y-4">
      <p class="opacity-90 text-center">
        Adicione programas à lista abaixo para bloquear ou desbloquear seu
        acesso à internet (entrada e saída).
      </p>

      <div
        class="grid grid-cols-1 md:grid-cols-2 gap-4 items-center p-4 bg-dark-blue-bg/30 rounded-lg"
      >
        <button
          on:click={selecionarArquivo}
          disabled={processando}
          class="w-full text-center px-4 py-3 bg-primary-purple hover:bg-primary-purple-dark rounded-lg transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
        >
          1. Selecionar Arquivo (.exe)...
        </button>
        <div class="relative">
          <select
            on:change={selecionarProgramaInstalado}
            disabled={processando}
            class="w-full px-4 py-3 bg-dark-blue-light border border-primary-purple rounded-md appearance-none disabled:opacity-60 disabled:cursor-not-allowed"
          >
            <option value="" disabled selected
              >2. Ou escolha um programa instalado...</option
            >
            {#each programasInstalados as prog}
              <option value={prog.caminho}>{prog.nome}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>

    <div class="flex-grow min-h-0 overflow-y-auto mt-4 pr-2 space-y-2">
      {#if executaveisParaAcao.length > 0}
        {#each executaveisParaAcao as exe}
          <div
            class="flex items-center gap-3 p-2 bg-dark-blue-bg/50 rounded-md text-sm"
          >
            <span
              class={`w-3 h-3 rounded-full flex-shrink-0 ${exe.bloqueado ? "bg-red-500" : "bg-green-500"}`}
              title={exe.bloqueado ? "Bloqueado" : "Não Bloqueado"}
            ></span>
            <span class="flex-grow truncate" title={exe.caminho}
              >{exe.caminho}</span
            >
            <button
              on:click={() => removerExecutavel(exe.caminho)}
              class="p-1 text-red-400 hover:text-red-200 flex-shrink-0"
              >&times;</button
            >
          </div>
        {/each}
      {:else}
        <div
          class="flex items-center justify-center h-full text-center opacity-60 italic"
        >
          Nenhum programa adicionado à lista.
        </div>
      {/if}
    </div>

    <div class="h-6 mt-2 text-center">
      {#if erro}<p class="text-red-500">{erro}</p>{/if}
      {#if mensagemSucesso}<p class="text-green-400">{mensagemSucesso}</p>{/if}
    </div>

    <div class="flex-shrink-0 grid grid-cols-2 gap-4 pt-2">
      <button
        on:click={() => executarAcao("bloquear")}
        disabled={processando || executaveisParaAcao.length === 0}
        class="w-full p-4 font-bold text-lg bg-red-600 hover:bg-red-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg"
      >
        Bloquear Selecionados
      </button>
      <button
        on:click={() => executarAcao("desbloquear")}
        disabled={processando || executaveisParaAcao.length === 0}
        class="w-full p-4 font-bold text-lg bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg"
      >
        Desbloquear Selecionados
      </button>
    </div>
  </div>

  <div class="flex-shrink-0 pt-4">
    <BotaoVoltar on:click={() => dispatch("voltar")} />
  </div>
</FeatureContainer>
