<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let tipo: "sucesso" | "erro" | "aviso" = "aviso";
  export let titulo: string;
  export let mensagem: string;
  export let textoConfirmar: string | null = null;
  export let textoCancelar: string | null = "Fechar";

  const dispatch = createEventDispatcher();

  const cores = {
    sucesso: "bg-green-700/20 border-green-700/30 text-green-500",
    erro: "bg-red-700/20 border-red-700/30 text-red-500",
    aviso: "bg-yellow-600/20 border-yellow-600/30 text-yellow-400",
  };
</script>

<div
  class="fixed inset-0 bg-black bg-opacity-60 backdrop-blur-sm flex items-center justify-center z-[1000] animate-modalFadeIn"
  on:click={() => dispatch("cancelar")}
  on:keydown={(e) => e.key === "Escape" && dispatch("cancelar")}
  role="button"
  tabindex="0"
>
  <div
    class="bg-dark-blue-light bg-opacity-95 border border-gray-700 rounded-xl w-11/12 max-w-lg max-h-[80vh] overflow-hidden animate-modalSlideIn shadow-2xl"
    on:click|stopPropagation
    on:keydown|stopPropagation
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div
      class="flex items-center p-5 gap-4 border-b border-gray-800 {cores[tipo]}"
    >
      <div class="flex-shrink-0 {cores[tipo]}">
        {#if tipo === "sucesso"}
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="3"><polyline points="20,6 9,17 4,12" /></svg
          >
        {:else if tipo === "erro"}
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            ><circle cx="12" cy="12" r="10" /><line
              x1="15"
              y1="9"
              x2="9"
              y2="15"
            /><line x1="9" y1="9" x2="15" y2="15" /></svg
          >
        {:else}
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            ><path
              d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
            ></path><line x1="12" y1="9" x2="12" y2="13"></line><line
              x1="12"
              y1="17"
              x2="12.01"
              y2="17"
            ></line></svg
          >
        {/if}
      </div>
      <h3 class="flex-grow m-0 text-lg font-semibold text-text-light">
        {titulo}
      </h3>
    </div>
    <div class="p-5">
      <p class="m-0 text-gray-200 leading-normal text-base">{mensagem}</p>
    </div>
    <div class="px-5 pb-5 flex justify-end gap-3">
      {#if textoCancelar}
        <button
          class="bg-dark-blue-bg text-white border border-gray-600 py-2.5 px-5 rounded-md text-base font-medium cursor-pointer transition-all duration-200 hover:bg-gray-700"
          on:click={() => dispatch("cancelar")}>{textoCancelar}</button
        >
      {/if}
      {#if textoConfirmar}
        <button
          class="bg-accent-orange text-dark-blue-bg border-none py-2.5 px-5 rounded-md text-base font-bold cursor-pointer transition-all duration-200 hover:brightness-110"
          on:click={() => dispatch("confirmar")}>{textoConfirmar}</button
        >
      {/if}
    </div>
  </div>
</div>

<style>
  @keyframes modalFadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes modalSlideIn {
    from {
      opacity: 0;
      transform: translateY(-20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .animate-modalFadeIn {
    animation: modalFadeIn 0.2s ease-out;
  }
  .animate-modalSlideIn {
    animation: modalSlideIn 0.3s ease-out;
  }
</style>
