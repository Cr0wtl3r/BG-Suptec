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

<div class="info-item-editavel">
  <span class="label">{label}</span>

  {#if !editando}
    <div class="display-container">
      <span class="value">{value}</span>
      <button
        class="btn-edit"
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
    <div class="edit-container">
      <input
        type="text"
        bind:value={valorEditavel}
        class="input-edit"
        on:keydown={(e) => e.key === "Enter" && salvar()}
        on:keydown={(e) => e.key === "Escape" && cancelar()}
      />
      <button class="btn-save" on:click={salvar} title="Salvar">
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
      <button class="btn-cancel" on:click={cancelar} title="Cancelar">
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

<style>
  .info-item-editavel {
    background-color: var(--bg-light);
    padding: 8px 12px;
    border-radius: 6px;
    border-left: 4px solid var(--accent-blue);
    display: flex;
    flex-direction: column;
  }
  .label {
    display: block;
    font-size: 0.7rem;
    opacity: 0.7;
    text-transform: uppercase;
  }
  .value {
    font-size: 0.9rem;
    font-weight: 600;
    word-break: break-all;
  }
  .display-container,
  .edit-container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .btn-edit,
  .btn-save,
  .btn-cancel {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-light);
    opacity: 0.6;
    transition: opacity 0.2s;
    padding: 2px;
  }
  .btn-edit:hover,
  .btn-save:hover,
  .btn-cancel:hover {
    opacity: 1;
  }
  .btn-edit[disabled] {
    cursor: not-allowed;
    opacity: 0.2;
  }
  .btn-save:hover {
    color: #4caf50;
  }
  .btn-cancel:hover {
    color: #f44336;
  }
  .input-edit {
    flex-grow: 1;
    background: #fff;
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 4px;
    color: #333;
    font-size: 0.9rem;
    font-weight: 600;
  }
</style>
