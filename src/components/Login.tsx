import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LoginProps {
  onLoginSucesso: () => void;
}

function Login({ onLoginSucesso }: LoginProps) {
  const [senha, setSenha] = useState("");
  const [mensagemErro, setMensagemErro] = useState("");
  const [carregando, setCarregando] = useState(false);

  async function fazerLogin() {
    setMensagemErro("");

    if (senha === "") {
      setMensagemErro("Por favor, digite a senha.");
      return;
    }

    setCarregando(true);
    try {
      const sucesso = await invoke<boolean>("login", { senha });
      if (sucesso) {
        onLoginSucesso();
      } else {
        setMensagemErro("Senha incorreta!");
      }
    } catch (err) {
      setMensagemErro(String(err));
    } finally {
      setCarregando(false);
    }
  }

  function handleKeyDown(event: React.KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Enter") {
      fazerLogin();
    }
  }

  return (
    <div className="relative z-20 mt-auto mb-auto w-11/12 max-w-md rounded-xl border border-gray-700 bg-dark-blue-light/70 p-10 text-center text-text-light shadow-2xl backdrop-blur-md">
      <img
        src="/profile.png"
        alt="Imagem de Perfil"
        className="mx-auto mb-5 h-24 w-24 rounded-full border-4 border-accent-orange bg-dark-blue-bg object-cover"
      />
      <h1 className="mb-2 text-3xl font-bold">Caixa de Ferramentas</h1>
      <p className="mb-5 opacity-80">Por favor, insira a senha para continuar.</p>
      <div className="mt-5 flex">
        <input
          id="senha"
          type="password"
          className="flex-grow rounded-l-md border border-transparent bg-dark-blue-bg/60 p-3 text-base text-text-light focus:border-structural-purple focus:outline-none focus:ring-2 focus:ring-structural-purple"
          value={senha}
          autoComplete="current-password"
          onChange={(e) => setSenha(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Senha"
          disabled={carregando}
        />
        <button
          type="button"
          className="cursor-pointer rounded-r-md border-none bg-accent-orange px-5 py-2 text-base font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-60"
          onClick={fazerLogin}
          disabled={carregando}
        >
          Entrar
        </button>
      </div>
      {mensagemErro && (
        <p className="mt-4 min-h-[20px] font-bold text-accent-orange">
          {mensagemErro}
        </p>
      )}
    </div>
  );
}

export default Login;
