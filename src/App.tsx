import { useState } from "react";
import "./App.css";
import Login from "./components/Login";
import PainelInformacoes from "./components/features/PainelInformacoes";
import AtivacaoWindows from "./components/features/AtivacaoWindows";
import AtivacaoOffice from "./components/features/AtivacaoOffice";
import LimpaCacheDNS from "./components/features/LimpaCacheDNS";
import LimpaSpoolImpressao from "./components/features/LimpaSpoolImpressao";
import DesativaHibernacao from "./components/features/DesativaHibernacao";
import AjustarHoraFormatacao from "./components/features/AjustarHoraFormatacao";

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
    ],
  },
  {
    nome: "Reparos e Soluções",
    funcionalidades: ["Ajustar Hora da Formatação"],
  },
];

function App() {
  const [logado, setLogado] = useState(false);
  const [funcionalidadeAtiva, setFuncionalidadeAtiva] = useState<string | null>(null);

  return (
    <main
      className="relative flex h-screen w-screen items-center justify-center overflow-hidden bg-dark-blue-bg bg-cover bg-center font-sans text-text-light"
      style={{ backgroundImage: "url('/background.jpg')" }}
    >
      <div className="absolute inset-0 bg-black/40" />
      {logado ? (
        <div className="relative z-10 h-[90vh] w-11/12 max-w-6xl">
          {funcionalidadeAtiva === "Ativação do Windows" ? (
            <AtivacaoWindows onVoltar={() => setFuncionalidadeAtiva(null)} />
          ) : funcionalidadeAtiva === "Ativação do Office" ? (
            <AtivacaoOffice onVoltar={() => setFuncionalidadeAtiva(null)} />
          ) : funcionalidadeAtiva === "Limpar Cache DNS" ? (
            <LimpaCacheDNS onVoltar={() => setFuncionalidadeAtiva(null)} />
          ) : funcionalidadeAtiva === "Limpar e Reiniciar Spool de Impressão" ? (
            <LimpaSpoolImpressao onVoltar={() => setFuncionalidadeAtiva(null)} />
          ) : funcionalidadeAtiva === "Desativar Hibernação do Windows" ? (
            <DesativaHibernacao onVoltar={() => setFuncionalidadeAtiva(null)} />
          ) : funcionalidadeAtiva === "Ajustar Hora da Formatação" ? (
            <AjustarHoraFormatacao onVoltar={() => setFuncionalidadeAtiva(null)} />
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
