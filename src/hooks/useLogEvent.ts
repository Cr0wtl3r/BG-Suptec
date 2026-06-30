import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

/**
 * Escuta um evento de log do Tauri (`eventName`) e chama `onLog` para cada
 * linha emitida. Substitui o `EventsOn`/`EventsOff` do Wails legado.
 *
 * `onLog` é mantido numa ref para que a inscrição não precise ser refeita
 * a cada render — apenas quando `eventName` muda — evitando perder eventos
 * emitidos entre um unlisten e um listen subsequente.
 */
export function useLogEvent(eventName: string, onLog: (message: string) => void) {
  const onLogRef = useRef(onLog);
  onLogRef.current = onLog;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelado = false;

    listen<string>(eventName, (event) => {
      onLogRef.current(event.payload);
    }).then((fn) => {
      if (cancelado) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelado = true;
      unlisten?.();
    };
  }, [eventName]);
}
