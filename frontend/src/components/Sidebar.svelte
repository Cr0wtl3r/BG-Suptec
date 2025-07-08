<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let logado = false;
  export let modulos = [];
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
  class="backdrop"
  role="button"
  tabindex="0"
  on:click={() => dispatch("close")}
  on:keydown={handleBackdropKeydown}
></div>

<aside class="sidebar">
  <div class="sidebar-content">
    <h2>Módulos</h2>
    <nav>
      {#each modulos as modulo}
        <div class="modulo-grupo">
          <h3>{modulo.nome}</h3>
          <ul>
            {#each modulo.funcionalidades as item}
              <li>
                <button
                  class="menu-item-button"
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
  .backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: rgba(0, 0, 0, 0.53);
    z-index: 40;
  }
  .sidebar {
    position: fixed;
    top: 0;
    right: 0;
    width: 350px;
    height: 100vh;
    z-index: 50;
    box-shadow: -10px 0 30px rgba(0, 0, 0, 0.553);
    transform: translateX(100%);
    animation: slideIn 0.3s forwards ease-out;
    background-color: rgba(9, 11, 55, 0.656);
    backdrop-filter: blur(10px);
    border-left: 1px solid rgba(240, 240, 240, 0.2);
  }
  .sidebar-content {
    width: 100%;
    height: 100%;
    padding: 20px;
    overflow-y: auto;
  }
  @keyframes slideIn {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
  h2 {
    text-align: center;
    color: var(--accent-orange);
    margin-bottom: 30px;
  }
  .modulo-grupo {
    margin-bottom: 25px;
  }
  .modulo-grupo h3 {
    margin: 0 0 10px 0;
    padding-bottom: 5px;
    border-bottom: 2px solid var(--accent-blue);
    font-size: 1.1rem;
  }
  .modulo-grupo ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .menu-item-button {
    display: block;
    width: 100%;
    background: none;
    border: none;
    color: var(--text-light);
    text-decoration: none;
    padding: 8px 10px;
    border-radius: 4px;
    transition: background-color 0.2s;
    font-family: inherit;
    font-size: inherit;
    text-align: left;
    cursor: pointer;
  }
  .menu-item-button:not(.disabled):hover {
    background-color: var(--bg-light);
  }
  .menu-item-button.disabled {
    color: #6c757d;
    cursor: not-allowed;
    opacity: 0.6;
  }
</style>
