/**
 * Nomes de eventos Tauri usados para streaming de logs ao frontend,
 * substituindo as strings mágicas espalhadas pelo `EventsOn`/`EventsEmit`
 * legados do Wails. Espelha as constantes em `src-tauri/src/events.rs`.
 */
export const EVENTOS = {
  logAtivacaoWindows: "log:ativacao:windows",
  ativacaoWindowsFinalizado: "ativacao:windows:finalizado",
  logAtivacaoOffice: "log:ativacao:office",
  ativacaoOfficeFinalizado: "ativacao:office:finalizado",
  logAjustarHoraFormatacao: "log:ajustar:hora:formatacao",
  logCompartilhamento: "log:compartilhamento",
  compartilhamentoFinalizado: "compartilhamento:finalizado",
  logAtivarProtecao: "log:ativar:protecao",
  ativarProtecaoFinalizado: "ativar:protecao:finalizado",
  logRestaurarPhotoviewer: "log:restaurar:photoviewer",
  restaurarPhotoviewerFinalizado: "log:restaurar:photoviewer:finalizado",
} as const;
