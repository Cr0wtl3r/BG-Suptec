<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let logado: boolean = false;
  export let modulos: { nome: string; funcionalidades: string[] }[] = [];
  const dispatch = createEventDispatcher();

  function navegar(item: string) {
    if (!logado) return;
    dispatch("navigate", item);
  }

  function handleBackdropKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      dispatch("close");
    }
  }
</script>

<div
  class="fixed inset-0 bg-black bg-opacity-50 z-40"
  role="button"
  tabindex="0"
  on:click={() => dispatch("close")}
  on:keydown={handleBackdropKeydown}
></div>

<aside
  class="fixed top-0 right-0 w-80 h-screen z-50 shadow-xl transform translate-x-full animate-slideIn bg-dark-blue-light bg-opacity-80 backdrop-blur-md border-l border-gray-700 overflow-y-auto"
>
  <div class="p-5">
    <h2 class="text-center text-accent-orange text-2xl font-bold mb-8">
      Módulos
    </h2>
    <nav>
      {#each modulos as modulo}
        <div class="mb-6">
          <h3
            class="mt-0 mb-2 pb-1 border-b-2 border-primary-purple text-lg font-semibold text-text-light"
          >
            {modulo.nome}
          </h3>
          <ul>
            {#each modulo.funcionalidades as item}
              <li>
                <button
                  class="block w-full bg-transparent border-none text-text-light py-2 px-3 rounded-md transition-colors duration-200 text-base text-left cursor-pointer
                    hover:bg-dark-blue-bg
                    disabled:text-gray-500 disabled:cursor-not-allowed disabled:opacity-60"
                  class:disabled={!logado}
                  disabled={!logado}
                  on:click={() => navegar(item)}
                >
                  {item}
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    </nav>
  </div>
</aside>

<style>
  @keyframes slideIn {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
  .animate-slideIn {
    animation-name: slideIn;
    animation-duration: 0.3s;
    animation-fill-mode: forwards;
    animation-timing-function: ease-out;
  }
</style>
