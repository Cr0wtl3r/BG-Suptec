import { useState } from "react";
import "./App.css";
import Login from "./components/Login";
import Sidebar from "./components/Sidebar";
import MainView from "./components/MainView";
import PainelInformacoes from "./components/features/PainelInformacoes";

const MODULOS = [
  {
    nome: "Ativação",
    funcionalidades: ["Ativação do Windows", "Ativação do Office"],
  },
  {
    nome: "Manutenção e Limpeza",
    funcionalidades: [
      "Limpar Cache DNS",
      "Limpar e Reiniciar Spool de Impressão",
      "Desativar Hibernação do Windows",
      "Limpeza Avançada",
    ],
  },
  {
    nome: "Reparos e Soluções",
    funcionalidades: [
      "Ajustar Hora da Formatação",
      "Corrigir Compartilhamento de Rede",
      "Ativar Proteção do Sistema",
      "Bloqueador de Programas no Firewall",
      "Restaurar Visualizador de Fotos",
      "Reparos do Windows",
      "Reset AnyDesk",
      "Conclusão de Formatação",
    ],
  },
  {
    nome: "Personalização e Sistema",
    funcionalidades: [
      "Ativar Gpedit.msc (Home)",
      "Alterar Layout do Teclado",
      "Políticas do Windows",
      "Disco e Edição do Windows",
      "Office Click-to-Run",
    ],
  },
  {
    nome: "Gerenciador de Energia",
    funcionalidades: ["Agendar Desligamento"],
  },
];

function App() {
  const [logado, setLogado] = useState(false);
  const [funcionalidadeAtiva, setFuncionalidadeAtiva] = useState<string | null>(null);
  const [menuAberto, setMenuAberto] = useState(false);

  function navegar(item: string) {
    setFuncionalidadeAtiva(item);
    setMenuAberto(false);
  }

  function handleMenuKeyDown(event: React.KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      setMenuAberto((aberto) => !aberto);
    }
  }

  return (
    <main
      className="relative flex h-screen w-screen items-center justify-center overflow-hidden bg-dark-blue-bg bg-cover bg-center font-sans text-text-light"
      style={{ backgroundImage: "url('/background.jpg')" }}
    >
      <div className="absolute inset-0 bg-black/40" />

      <button
        className="absolute top-5 right-5 z-50 flex h-8 w-8 cursor-pointer flex-col justify-between gap-1.5 border-none bg-transparent p-2 outline-none hover:opacity-80 focus:outline-2 focus:outline-offset-2 focus:outline-white"
        onClick={() => setMenuAberto((aberto) => !aberto)}
        onKeyDown={handleMenuKeyDown}
        tabIndex={0}
        aria-label="Abrir menu"
      >
        <div className="h-0.5 w-full rounded bg-white transition-all duration-300" />
        <div className="h-0.5 w-full rounded bg-white transition-all duration-300" />
        <div className="h-0.5 w-full rounded bg-white transition-all duration-300" />
      </button>

      {menuAberto && (
        <Sidebar
          logado={logado}
          modulos={MODULOS}
          onNavigate={navegar}
          onClose={() => setMenuAberto(false)}
        />
      )}

      {logado ? (
        <div className="relative z-10 h-[90vh] w-11/12 max-w-6xl">
          {funcionalidadeAtiva ? (
            <MainView
              visao={funcionalidadeAtiva}
              onVoltar={() => setFuncionalidadeAtiva(null)}
            />
          ) : (
            <PainelInformacoes modulos={MODULOS} onNavigate={setFuncionalidadeAtiva} />
          )}
        </div>
      ) : (
        <Login onLoginSucesso={() => setLogado(true)} />
      )}
    </main>
  );
}

export default App;
