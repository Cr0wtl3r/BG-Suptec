import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";

interface AgendarDesligamentoProps {
  onVoltar: () => void;
}

function AgendarDesligamento({ onVoltar }: AgendarDesligamentoProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);
  const [tempoSelecionado, setTempoSelecionado] = useState("60");
  const [agendamentoAtivo, setAgendamentoAtivo] = useState(false);

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  async function iniciar() {
    setEmExecucao(true);
    setAgendamentoAtivo(false);
    setLogLines(["Iniciando agendamento de desligamento..."]);

    try {
      adicionarLog("Verificando e cancelando qualquer agendamento anterior...");
      await invoke("cancelar_desligamento");
      adicionarLog("Agendamento anterior cancelado (se existia).");
      await new Promise((resolve) => setTimeout(resolve, 500));

      adicionarLog(`Agendando desligamento em ${tempoSelecionado} segundos...`);
      const resultado = await invoke<string>("agendar_desligamento", {
        segundos: Number(tempoSelecionado),
      });
      adicionarLog("Resultado do comando: " + resultado);
      adicionarLog(`Seu computador será desligado em ${tempoSelecionado} segundos.`);
      setAgendamentoAtivo(true);
    } catch (err) {
      adicionarLog(`ERRO: ${err}`);
      setAgendamentoAtivo(false);
    } finally {
      adicionarLog("--- Operação de agendamento concluída ---");
      setEmExecucao(false);
    }
  }

  async function cancelarDesligamento() {
    setEmExecucao(true);
    setLogLines(["Tentando cancelar o desligamento agendado..."]);
    try {
      adicionarLog("Executando comando: shutdown /a");
      const resultado = await invoke<string>("cancelar_desligamento");
      adicionarLog("Resultado do comando: " + resultado);
      adicionarLog("Desligamento agendado cancelado com sucesso!");
      setAgendamentoAtivo(false);
    } catch (err) {
      adicionarLog(`ERRO ao cancelar: ${err}`);
    } finally {
      adicionarLog("--- Operação de cancelamento concluída ---");
      setEmExecucao(false);
    }
  }

  return (
    <FeatureContainer titulo="Agendar Desligamento">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="mx-auto w-full max-w-xl flex-shrink-0 pr-2">
          <p className="mb-6 mt-0 text-center opacity-90">
            Agende o desligamento automático do seu computador após um período de tempo. Você pode
            cancelar o desligamento agendado a qualquer momento.
          </p>

          <div className="mb-2">
            <label htmlFor="tempo-desligamento" className="mb-2 block font-bold text-text-light">
              Selecione o tempo para desligar (segundos):
            </label>
            <select
              id="tempo-desligamento"
              value={tempoSelecionado}
              onChange={(e) => setTempoSelecionado(e.target.value)}
              disabled={emExecucao || agendamentoAtivo}
              className="w-full rounded-md border border-structural-purple bg-dark-blue-light py-2 text-base text-text-light focus:outline-none focus:ring-1 focus:ring-structural-purple disabled:cursor-not-allowed disabled:opacity-60"
            >
              <option value="60">1 minuto (60s)</option>
              <option value="300">5 minutos (300s)</option>
              <option value="600">10 minutos (600s)</option>
              <option value="1800">30 minutos (1800s)</option>
              <option value="3600">1 hora (3600s)</option>
              <option value="7200">2 horas (7200s)</option>
              <option value="18000">5 horas (18000s)</option>
            </select>
          </div>

          <button
            className="mt-4 w-full cursor-pointer rounded-lg border-none bg-accent-orange p-4 text-xl font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
            onClick={iniciar}
            disabled={emExecucao}
          >
            {agendamentoAtivo ? "Desligamento Agendado..." : "Agendar Desligamento"}
          </button>

          {agendamentoAtivo && (
            <button
              className="mx-auto mt-2 block cursor-pointer rounded-lg border-none bg-red-600 px-4 py-2 text-base font-semibold text-white transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-60"
              onClick={cancelarDesligamento}
              disabled={emExecucao}
            >
              Cancelar Desligamento Agendado
            </button>
          )}
        </div>

        <LogPanel logLines={logLines} />
      </div>

      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>
    </FeatureContainer>
  );
}

export default AgendarDesligamento;
