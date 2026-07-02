import type { ComponentType } from "react";
import AtivacaoWindows from "./features/AtivacaoWindows";
import AtivacaoOffice from "./features/AtivacaoOffice";
import LimpaCacheDNS from "./features/LimpaCacheDNS";
import LimpaSpoolImpressao from "./features/LimpaSpoolImpressao";
import DesativaHibernacao from "./features/DesativaHibernacao";
import AjustarHoraFormatacao from "./features/AjustarHoraFormatacao";
import CorrigirCompartilhamento from "./features/CorrigirCompartilhamento";
import AtivarProtecaoSistema from "./features/AtivarProtecaoSistema";
import BloqueadorFirewall from "./features/BloqueadorFirewall";
import RestaurarPhotoViewer from "./features/RestaurarPhotoViewer";
import AlterarLayoutTeclado from "./features/AlterarLayoutTeclado";
import AtivarGpedit from "./features/AtivarGpedit";
import AgendarDesligamento from "./features/AgendarDesligamento";
import PoliticasWindows from "./features/PoliticasWindows";
import ReparosWindows from "./features/ReparosWindows";
import DiscoWindows from "./features/DiscoWindows";
import LimpezaAvancada from "./features/LimpezaAvancada";
import ResetAnyDesk from "./features/ResetAnyDesk";
import OfficeC2R from "./features/OfficeC2R";
import ConclusaoFormatacao from "./features/ConclusaoFormatacao";

interface FeatureProps {
  onVoltar: () => void;
}

const COMPONENTES: Record<string, ComponentType<FeatureProps>> = {
  "Ativação do Windows": AtivacaoWindows,
  "Ativação do Office": AtivacaoOffice,
  "Limpar Cache DNS": LimpaCacheDNS,
  "Limpar e Reiniciar Spool de Impressão": LimpaSpoolImpressao,
  "Desativar Hibernação do Windows": DesativaHibernacao,
  "Ajustar Hora da Formatação": AjustarHoraFormatacao,
  "Corrigir Compartilhamento de Rede": CorrigirCompartilhamento,
  "Ativar Proteção do Sistema": AtivarProtecaoSistema,
  "Bloqueador de Programas no Firewall": BloqueadorFirewall,
  "Restaurar Visualizador de Fotos": RestaurarPhotoViewer,
  "Alterar Layout do Teclado": AlterarLayoutTeclado,
  "Ativar Gpedit.msc (Home)": AtivarGpedit,
  "Agendar Desligamento": AgendarDesligamento,
  "Políticas do Windows": PoliticasWindows,
  "Reparos do Windows": ReparosWindows,
  "Disco e Edição do Windows": DiscoWindows,
  "Limpeza Avançada": LimpezaAvancada,
  "Reset AnyDesk": ResetAnyDesk,
  "Office Click-to-Run": OfficeC2R,
  "Conclusão de Formatação": ConclusaoFormatacao,
};

interface MainViewProps {
  visao: string;
  onVoltar: () => void;
}

function MainView({ visao, onVoltar }: MainViewProps) {
  const Componente = COMPONENTES[visao];

  if (!Componente) {
    return <p>Erro: funcionalidade "{visao}" não encontrada.</p>;
  }

  return <Componente onVoltar={onVoltar} />;
}

export default MainView;
