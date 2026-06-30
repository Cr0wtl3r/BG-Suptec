<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let label: string;
  export let value: string;
  export let disabled: boolean = false;

  let editando = false;
  let valorEditavel = value;

  const dispatch = createEventDispatcher();

  function ativarEdicao() {
    if (disabled) return;
    valorEditavel = value;
    editando = true;
  }

  function salvar() {
    dispatch("save", valorEditavel);
    editando = false;
  }

  function cancelar() {
    editando = false;
  }
</script>

<div
  class="bg-dark-blue-bg p-3 rounded-md border-l-4 border-primary-purple flex flex-col"
>
  <span class="block text-xs opacity-70 uppercase mb-1 text-center"
    >{label}</span
  >

  {#if !editando}
    <div class="relative flex items-center justify-center min-h-[28px]">
      <span class="text-sm font-semibold break-all text-center px-5"
        >{value}</span
      >
      <button
        aria-label="Editar campo"
        class="absolute right-0 top-1/2 -translate-y-1/2 bg-transparent border-none cursor-pointer text-text-light opacity-60 transition-opacity duration-200 p-0.5 hover:opacity-100 hover:text-accent-orange disabled:cursor-not-allowed disabled:opacity-20"
        on:click={ativarEdicao}
        title="Editar"
        {disabled}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"
          ></path></svg
        >
      </button>
    </div>
  {:else}
    <div class="flex items-center gap-2">
      <input
        type="text"
        bind:value={valorEditavel}
        class="flex-grow bg-dark-blue-light border border-primary-purple rounded p-1.5 text-text-light text-sm font-semibold text-center"
        on:keydown={(e) => e.key === "Enter" && salvar()}
        on:keydown={(e) => e.key === "Escape" && cancelar()}
      />
      <button
        aria-label="Salvar edição"
        class="bg-transparent border-none cursor-pointer text-text-light opacity-60 transition-opacity duration-200 p-0.5
        hover:opacity-100 hover:text-green-500"
        on:click={salvar}
        title="Salvar"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><polyline points="20 6 9 17 4 12"></polyline></svg
        >
      </button>
      <button
        aria-label="Cancelar edição"
        class="bg-transparent border-none cursor-pointer text-text-light opacity-60 transition-opacity duration-200 p-0.5
        hover:opacity-100 hover:text-red-500"
        on:click={cancelar}
        title="Cancelar"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><line x1="18" y1="6" x2="6" y2="18"></line><line
            x1="6"
            y1="6"
            x2="18"
            y2="18"
          ></line></svg
        >
      </button>
    </div>
  {/if}
</div>
