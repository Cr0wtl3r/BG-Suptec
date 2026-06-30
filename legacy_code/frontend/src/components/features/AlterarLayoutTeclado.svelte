<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    ObterLayoutsDisponiveis,
    ObterLayoutAtivo,
    AlterarLayoutDeTeclado,
  } from "../../../wailsjs/go/main/App";
  import FeatureContainer from "../shared/FeatureContainer.svelte";
  import BotaoVoltar from "../shared/BotaoVoltar.svelte";
  import TecladoABNT2 from "../shared/teclados/TecladoABNT2.svelte";
  import TecladoUS from "../shared/teclados/TecladoUS.svelte";
  import TecladoES from "../shared/teclados/TecladoES.svelte";

  type TecladoInfo = {
    id: string;
    nome: string;
    tagIdioma: string;
  };

  const dispatch = createEventDispatcher();

  let todosLayouts: TecladoInfo[] = [];
  let nomeLayoutAtual = "Carregando...";
  let idLayoutAtivo = "";
  let layoutSelecionado: string | undefined = undefined;

  let erro = "";
  let salvando = false;
  let mensagemSucesso = "";

  onMount(async () => {
    try {
      todosLayouts = await ObterLayoutsDisponiveis();
      idLayoutAtivo = await ObterLayoutAtivo();
      const ativo = todosLayouts.find((l) => l.id === idLayoutAtivo);
      if (ativo) {
        nomeLayoutAtual = ativo.nome;
        layoutSelecionado = ativo.id;
      } else {
        nomeLayoutAtual = `Não reconhecido (${idLayoutAtivo})`;
        if (todosLayouts.length > 0) {
          layoutSelecionado = todosLayouts[0].id;
        }
      }
    } catch (e) {
      erro = `Erro ao carregar layouts: ${e}`;
      nomeLayoutAtual = "Erro na detecção";
    }
  });

  async function aplicarLayout() {
    if (!layoutSelecionado) return;
    const layoutParaAplicar = todosLayouts.find(
      (l) => l.id === layoutSelecionado,
    );
    if (!layoutParaAplicar) {
      erro = "Erro interno: Layout selecionado não encontrado na lista.";
      return;
    }

    salvando = true;
    mensagemSucesso = "";
    erro = "";
    try {
      await AlterarLayoutDeTeclado(layoutParaAplicar.tagIdioma);
      mensagemSucesso = "Layout do teclado alterado com sucesso!";

      await new Promise((resolve) => setTimeout(resolve, 1000));

      idLayoutAtivo = await ObterLayoutAtivo();
      const ativo = todosLayouts.find((l) => l.id === idLayoutAtivo);
      nomeLayoutAtual = ativo
        ? ativo.nome
        : `Não reconhecido (${idLayoutAtivo})`;
    } catch (e) {
      erro = `Falha ao aplicar layout: ${e}`;
    } finally {
      salvando = false;
    }
  }

  $: tecladoComponente = (() => {
    if (!layoutSelecionado) return null;
    if (layoutSelecionado.includes("0416")) return TecladoABNT2;
    if (layoutSelecionado.includes("0816")) return TecladoABNT2;
    if (layoutSelecionado.includes("0409")) return TecladoUS;
    if (
      layoutSelecionado.includes("0c0a") ||
      layoutSelecionado.includes("080a")
    )
      return TecladoES;
    return null;
  })();
</script>

<FeatureContainer titulo="Alterar Layout do Teclado">
  <div class="flex-grow min-h-0 overflow-y-auto pr-2">
    <div class="space-y-6">
      <div class="p-1 bg-dark-blue-bg/50 rounded-lg text-center">
        <h3 class="text-sm uppercase opacity-70 mb-1">
          Layout do Teclado Ativo no Sistema
        </h3>
        <p class="text-xl font-bold text-accent-orange">{nomeLayoutAtual}</p>
      </div>

      <div>
        <label for="layout-select" class="block font-bold mb-2"
          >Selecione um layout para visualizar ou aplicar:</label
        >
        <div class="flex flex-col sm:flex-row gap-2">
          <select
            id="layout-select"
            class="w-full flex-grow p-3 bg-dark-blue-light text-text-light border border-primary-purple rounded-md text-base focus:outline-none focus:ring-1 focus:ring-primary-purple"
            bind:value={layoutSelecionado}
            disabled={salvando || todosLayouts.length === 0}
          >
            {#each todosLayouts as layout}
              <option value={layout.id}>
                {layout.nome}{layout.id === idLayoutAtivo ? " (Atual)" : ""}
              </option>
            {/each}
          </select>
          <button
            class="px-6 py-3 text-lg font-bold cursor-pointer bg-accent-orange text-dark-blue-bg border-none rounded-lg transition-all duration-200 hover:brightness-110 disabled:bg-gray-600 disabled:cursor-not-allowed"
            on:click={aplicarLayout}
            disabled={salvando ||
              !layoutSelecionado ||
              layoutSelecionado === idLayoutAtivo}
          >
            {salvando ? "Aplicando..." : "Aplicar"}
          </button>
        </div>
        {#if erro}
          <p class="text-red-500 mt-2 text-center">{erro}</p>
        {/if}
        {#if mensagemSucesso}
          <p class="text-green-400 mt-2 text-center">{mensagemSucesso}</p>
        {/if}
      </div>

      <div class="grid md:grid-cols-2 gap-6 items-center">
        <div class="w-full">
          <label for="test-area" class="block font-bold mb-2"
            >Teste sua digitação aqui:</label
          >
          <textarea
            id="test-area"
            rows="5"
            class="w-full p-3 bg-dark-blue-bg/70 border border-primary-purple/50 rounded-lg text-base focus:outline-none focus:ring-1 focus:ring-primary-purple"
            placeholder="Teste teclas como ´ ` ~ ^ ç ; / . ,"
          ></textarea>
        </div>

        <div class="w-full">
          <h3 class="block font-bold mb-2 text-center">Visualização</h3>
          <div
            class="p-3 bg-dark-blue-bg/30 rounded-lg min-h-[100px] flex items-center justify-center"
          >
            {#if tecladoComponente}
              <svelte:component this={tecladoComponente} />
            {:else}
              <p class="text-center opacity-70 italic">
                Nenhuma visualização disponível para este layout.
              </p>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>

  <div class="flex-shrink-0 pt-4">
    <BotaoVoltar on:click={() => dispatch("voltar")} />
  </div>
</FeatureContainer>
