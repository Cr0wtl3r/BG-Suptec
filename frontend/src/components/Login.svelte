<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { Login as GoLogin } from "../../wailsjs/go/main/App";

  const dispatch = createEventDispatcher();
  let senha = "";
  let mensagemErro = "";

  async function fazerLogin() {
    mensagemErro = "";
    if (senha === "") {
      mensagemErro = "Por favor, digite a senha.";
      return;
    }
    try {
      const sucesso = await GoLogin(senha);
      if (sucesso) {
        dispatch("loginsucesso");
      } else {
        mensagemErro = "Senha incorreta!";
      }
    } catch (err) {
      mensagemErro = `Erro inesperado: ${err}`;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      fazerLogin();
    }
  }
</script>

<div class="login-box">
  <img src="/profile.png" alt="Imagem de Perfil" class="profile-pic" />
  <h1>Caixa de Ferramentas</h1>
  <p class="subtitle">Por favor, insira a senha para continuar.</p>
  <div class="input-group">
    <input
      id="senha"
      type="password"
      class="input"
      bind:value={senha}
      autocomplete="current-password"
      on:keydown={handleKeydown}
      placeholder="Senha"
    />
    <button class="btn" on:click={fazerLogin}>Entrar</button>
  </div>
  {#if mensagemErro}
    <p class="error">{mensagemErro}</p>
  {/if}
</div>

<style>
  .login-box {
    position: relative;
    z-index: 2;
    background-color: rgba(25, 28, 89, 0.2);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(240, 240, 240, 0.301);
    padding: 40px;
    border-radius: 12px;
    color: var(--text-light);
    text-align: center;
    width: 90%;
    max-width: 400px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.726);
  }
  .profile-pic {
    width: 100px;
    height: 100px;
    border-radius: 50%;
    object-fit: cover;
    border: 3px solid var(--accent-orange);
    margin-bottom: 20px;
    background-color: var(--bg-light);
  }
  .subtitle {
    opacity: 0.8;
  }
  .input-group {
    display: flex;
    margin-top: 20px;
  }
  .input {
    flex-grow: 1;
    padding: 12px;
    border: 1px solid transparent;
    background-color: rgba(22, 62, 115, 0.5);
    color: var(--text-light);
    border-radius: 4px 0 0 4px;
    font-size: 16px;
  }
  .input:focus {
    outline: none;
    border-color: var(--accent-blue);
    box-shadow: 0 0 5px var(--accent-blue);
  }
  .btn {
    padding: 10px 20px;
    border: none;
    background-color: var(--accent-orange);
    color: var(--text-light);
    cursor: pointer;
    border-radius: 0 4px 4px 0;
    font-weight: bold;
    font-size: 16px;
    transition: filter 0.2s;
  }
  .btn:hover {
    filter: brightness(1.1);
  }
  .error {
    color: var(--accent-orange);
    margin-top: 15px;
    font-weight: bold;
    min-height: 20px;
  }
</style>
