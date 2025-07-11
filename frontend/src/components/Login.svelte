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

<div
  class="relative z-20 bg-dark-blue-light bg-opacity-70 backdrop-blur-md border border-gray-700 p-10 rounded-xl text-text-light text-center w-11/12 max-w-md shadow-2xl **mt-auto mb-auto**"
>
  <img
    src="/profile.png"
    alt="Imagem de Perfil"
    class="w-24 h-24 rounded-full object-cover border-4 border-accent-orange mb-5 mx-auto bg-dark-blue-bg"
  />
  <h1 class="text-3xl font-bold mb-2">Caixa de Ferramentas</h1>
  <p class="opacity-80 mb-5">Por favor, insira a senha para continuar.</p>
  <div class="flex mt-5">
    <input
      id="senha"
      type="password"
      class="flex-grow p-3 border border-transparent bg-dark-blue-bg bg-opacity-60 text-text-light rounded-l-md text-base focus:outline-none focus:border-primary-purple focus:ring-2 focus:ring-primary-purple"
      bind:value={senha}
      autocomplete="current-password"
      on:keydown={handleKeydown}
      placeholder="Senha"
    />
    <button
      class="px-5 py-2 border-none bg-accent-orange text-dark-blue-bg cursor-pointer rounded-r-md font-bold text-base transition-all duration-200 hover:brightness-110"
      on:click={fazerLogin}>Entrar</button
    >
  </div>
  {#if mensagemErro}
    <p class="text-accent-orange mt-4 font-bold min-h-[20px]">{mensagemErro}</p>
  {/if}
</div>
