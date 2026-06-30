# Product

## Register

product

## Users

Técnicos de TI da equipe interna de suporte. Executam o BG-SupTec diretamente na máquina Windows que estão atendendo (própria da empresa ou de um cliente), com privilégios de administrador, para realizar manutenção técnica padronizada: ativação de Windows/Office, diagnóstico e reparo de rede, limpeza de cache/spool, bloqueio de programas no firewall, restauração de configurações do sistema, entre outras. O app é protegido por senha e roda localmente — não é uma ferramenta web nem multi-tenant.

## Product Purpose

Centralizar e padronizar procedimentos técnicos que hoje seriam executados manualmente via terminal/PowerShell/registro, reduzindo erro humano e tempo de atendimento. Cada funcionalidade é uma ação encapsulada, tipada e auditável — não um terminal genérico. Sucesso significa: o técnico executa a ação certa, vê o resultado em tempo real (log/sucesso/erro) e tem confiança de que nada foi feito às escuras, mesmo em ações potencialmente destrutivas (alterar IP, desbloquear firewall, editar registro).

## Brand Personality

Técnico e direto, porém visualmente moderno e polido. Não é uma ferramenta austera de linha de comando, mas também não busca encantar como um produto de consumo — a estética serve a tarefa: hierarquia clara, densidade de informação controlada, feedback imediato. O verniz moderno (superfícies translúcidas, profundidade, microanimações) comunica que a ferramenta é cuidada e confiável, não descartável.

## Anti-references

- Painel administrativo genérico estilo CRUD (tabelas cinzas, cards idênticos, zero identidade visual)
- Excesso de gradientes, texto com gradiente, eyebrows numéricos decorativos, glassmorphism sem propósito — clichês de "IA generativa"
- Estética de terminal hacker/cyberpunk (verde sobre preto, fonte monoespaçada agressiva) — apesar de manipular registro e firewall, o tom não é de "ferramenta de invasão"

## Design Principles

1. **Clareza antes de estética** — em ações destrutivas (firewall, registro, ativação, rede), a hierarquia visual deve deixar óbvio o que vai acontecer antes do técnico confirmar.
2. **Densidade técnica sem ruído** — informação de sistema (specs, IPs, logs, status) precisa caber na tela sem virar bagunça visual.
3. **Confiança através de precisão** — toda ação tem feedback explícito (log em tempo real, sucesso/erro/aviso); nunca silêncio ou ambiguidade sobre o que aconteceu.
4. **Modernidade funcional, não decorativa** — o visual contemporâneo (blur, profundidade, microanimações) reforça hierarquia e estado; nunca é estética pela estética.
5. **Consistência entre as 15 features** — o mesmo padrão (FeatureContainer + LogPanel + Modal) se repete; nenhuma tela reinventa sua própria linguagem visual.

## Accessibility & Inclusion

Sem exigência formal de conformidade WCAG (ferramenta interna, uso por técnicos treinados). Ainda assim, manter contraste de texto adequado e estados de foco visíveis por boa prática, já que o app frequentemente é operado sob pressão (atendimento ao vivo, cliente observando a tela).
