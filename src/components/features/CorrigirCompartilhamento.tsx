import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import FeatureContainer from "../shared/FeatureContainer";
import LogPanel from "../shared/LogPanel";
import BotaoVoltar from "../shared/BotaoVoltar";
import Modal from "../shared/Modal";
import { useLogEvent } from "../../hooks/useLogEvent";
import { EVENTOS } from "../../lib/events";

interface CorrigirCompartilhamentoProps {
  onVoltar: () => void;
}

function CorrigirCompartilhamento({ onVoltar }: CorrigirCompartilhamentoProps) {
  const [logLines, setLogLines] = useState<string[]>(["Aguardando início..."]);
  const [emExecucao, setEmExecucao] = useState(false);
  const [mostrarModalAviso, setMostrarModalAviso] = useState(false);
  const [mostrarModalReinicio, setMostrarModalReinicio] = useState(false);

  function adicionarLog(mensagem: string) {
    const timestamp = new Date().toLocaleTimeString("pt-BR");
    setLogLines((prev) => [...prev, `[${timestamp}] ${mensagem}`]);
  }

  useLogEvent(EVENTOS.logCompartilhamento, adicionarLog);

  async function iniciar() {
    setMostrarModalAviso(false);
    setEmExecucao(true);
    setLogLines([]);

    let unlisten: (() => void) | undefined;
    const promessaDeConclusao = new Promise<void>((resolve) => {
      listen(EVENTOS.compartilhamentoFinalizado, () => resolve()).then((fn) => {
        unlisten = fn;
      });
    });

    try {
      await invoke("corrigir_compartilhamento");
      await promessaDeConclusao;
      setMostrarModalReinicio(true);
    } catch (err) {
      adicionarLog(`ERRO CRÍTICO: ${err}`);
    } finally {
      unlisten?.();
      setEmExecucao(false);
    }
  }

  async function reiniciar() {
    try {
      adicionarLog("[INFO] Comando de reinicialização enviado.");
      await invoke("reiniciar_computador");
      setMostrarModalReinicio(false);
    } catch (err) {
      adicionarLog(`[ERRO] Não foi possível reiniciar: ${err}`);
      setMostrarModalReinicio(false);
    }
  }

  return (
    <FeatureContainer titulo="Corrigir Compartilhamento de Rede">
      <div className="flex min-h-0 flex-grow flex-col">
        <div className="mx-auto w-full max-w-xl flex-shrink-0 pr-2">
          <p className="mb-6 mt-0 text-center opacity-90">
            Aplica um conjunto completo de correções para problemas comuns de compartilhamento de
            pastas e impressoras em rede local. Requer privilégios de Administrador.
          </p>
          <button
            className="mt-4 w-full cursor-pointer rounded-lg border-none bg-accent-orange p-4 text-xl font-bold text-dark-blue-bg transition-all duration-200 hover:brightness-110 disabled:cursor-not-allowed disabled:bg-gray-600"
            onClick={() => setMostrarModalAviso(true)}
            disabled={emExecucao}
          >
            {emExecucao ? "Aplicando..." : "Aplicar Correções"}
          </button>
        </div>

        <LogPanel logLines={logLines} />
      </div>

      <div className="flex-shrink-0 pt-4">
        <BotaoVoltar onClick={onVoltar} />
      </div>

      {mostrarModalAviso && (
        <Modal
          tipo="aviso"
          titulo="Redução de Segurança SMB"
          mensagem="Esta correção desativa a exigência de assinatura digital SMB (RequireSecuritySignature) e permite logons de convidado com senha em branco (limitblankpassworduse), entre outros ajustes de registro, para restaurar a compatibilidade de compartilhamento. Isso reduz a segurança da rede local em troca de corrigir o compartilhamento. Deseja continuar?"
          textoConfirmar="Continuar"
          textoCancelar="Cancelar"
          onConfirmar={iniciar}
          onCancelar={() => setMostrarModalAviso(false)}
        />
      )}

      {mostrarModalReinicio && (
        <Modal
          tipo="aviso"
          titulo="Reinicialização Recomendada"
          mensagem="As correções foram aplicadas com sucesso. É altamente recomendável reiniciar o computador para que todas as alterações tenham efeito."
          textoConfirmar="Reiniciar Agora"
          textoCancelar="Depois"
          onConfirmar={reiniciar}
          onCancelar={() => setMostrarModalReinicio(false)}
        />
      )}
    </FeatureContainer>
  );
}

export default CorrigirCompartilhamento;
