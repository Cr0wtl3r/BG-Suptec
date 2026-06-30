---
name: BG-SupTec
description: Painel de controle de suporte técnico Windows — HUD de vidro flutuando sobre a máquina real
colors:
  accent-orange: "#ed5f0c"
  dark-blue-bg: "#1A202C"
  dark-blue-light: "#2D3748"
  text-light: "#EDF2F7"
  structural-purple: "#171958af"
  structural-purple-dim: "#201f1f70"
  state-success: "#15803d"
  state-error: "#b91c1c"
  state-warning: "#ca8a04"
typography:
  title:
    fontFamily: "Segoe UI, Tahoma, Geneva, Verdana, sans-serif"
    fontSize: "1.5rem"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "normal"
  heading:
    fontFamily: "Segoe UI, Tahoma, Geneva, Verdana, sans-serif"
    fontSize: "1.125rem"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "normal"
  body:
    fontFamily: "Segoe UI, Tahoma, Geneva, Verdana, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  label:
    fontFamily: "Segoe UI, Tahoma, Geneva, Verdana, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "0.05em"
  mono:
    fontFamily: "Consolas, ui-monospace, monospace"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
  xl: "12px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
  xl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.accent-orange}"
    textColor: "{colors.dark-blue-bg}"
    rounded: "{rounded.md}"
    padding: "10px 20px"
  button-primary-hover:
    backgroundColor: "{colors.accent-orange}"
    textColor: "{colors.dark-blue-bg}"
    rounded: "{rounded.md}"
    padding: "10px 20px"
  button-secondary:
    backgroundColor: "{colors.dark-blue-bg}"
    textColor: "{colors.text-light}"
    rounded: "{rounded.md}"
    padding: "10px 20px"
  panel-glass:
    backgroundColor: "{colors.dark-blue-light}"
    textColor: "{colors.text-light}"
    rounded: "{rounded.xl}"
    padding: "24px"
---

# Design System: BG-SupTec

## 1. Overview

**Creative North Star: "O Painel de Controle (HUD do Técnico)"**

BG-SupTec não é um dashboard abstrato — é um painel de controle de vidro que flutua sobre a máquina real sendo atendida. A foto de fundo (a estação de trabalho, com escurecimento de 40%) ancora cada tela ao contexto físico do atendimento; os painéis translúcidos com blur são instrumentos pousados sobre essa cena, não uma camada decorativa. A profundidade vem dessa relação entre fundo fotográfico e vidro — nunca de sombra pesada ou gradiente.

A personalidade é técnica e direta: hierarquia clara, uma única cor de destaque (laranja queimado) usada com extrema disciplina, feedback explícito para toda ação. O verniz moderno (blur, transições suaves, profundidade) existe para comunicar precisão e cuidado, nunca para impressionar. Este sistema rejeita explicitamente: painéis administrativos genéricos estilo CRUD (tabelas cinzas, cards idênticos), clichês de "IA generativa" (texto em gradiente, eyebrows decorativos, glassmorphism sem propósito) e a estética de terminal hacker/cyberpunk — mesmo manipulando registro e firewall, o tom é de instrumento de precisão, não de ferramenta de invasão.

**Key Characteristics:**
- Vidro sobre realidade: cada superfície de UI é translúcida sobre o fundo fotográfico da estação de trabalho, nunca opaca por padrão.
- Um único acento: laranja queimado (#ed5f0c) marca exclusivamente ação primária e estado de foco — nunca decoração.
- Feedback sempre visível: toda ação tem log, modal ou indicador de estado; silêncio nunca é uma resposta válida.
- Profundidade por camada, não por sombra: `backdrop-blur` + opacidade de superfície substituem `box-shadow` pesado.

## 2. Colors

Paleta restrita e disciplinada: dois tons estruturais de azul-ardósia, um único acento queimado, e cores de estado emprestadas do vocabulário semântico padrão (sucesso/erro/aviso) — nada além disso.

### Primary
- **Laranja Queimado** (#ed5f0c): único acento do sistema. Reservado para o botão de ação primária (Entrar, Confirmar), títulos de destaque (`FeatureContainer`), e o estado de hover de ícones interativos. Aparece em no máximo um elemento de destaque por tela.

### Neutral
- **Ardósia Profunda** (#1A202C — `dark-blue-bg`): fundo de base de toda a aplicação, por trás da foto da estação; também usado em campos de log e botões secundários.
- **Ardósia Clara** (#2D3748 — `dark-blue-light`): superfície dos painéis de vidro (cards, modais, accordions), quase sempre com opacidade reduzida (35–95%) e `backdrop-blur` para deixar o fundo fotográfico transparecer.
- **Branco-Gelo** (#EDF2F7 — `text-light`): cor de texto primária sobre as superfícies escuras.
- **Roxo Estrutural** (#171958af / #201f1f70 — translúcidos): usados apenas em bordas e cabeçalhos de accordion como articulação estrutural discreta, nunca como acento de destaque.

### Estado (semântico, Tailwind-padrão)
- **Sucesso** (#15803d / green-700): confirmações de ação concluída.
- **Erro** (#b91c1c / red-700): falhas de execução e validação.
- **Aviso** (#ca8a04 / yellow-600): confirmações que exigem atenção antes de prosseguir (ex.: ações destrutivas).

### Named Rules
**A Regra do Acento Único.** O laranja queimado é a única cor com função de "chame minha atenção" no sistema. Se duas coisas competem por destaque na mesma tela, uma delas está usando a cor errada.

**A Regra do Vidro sobre Realidade.** Nenhuma superfície de conteúdo é opaca por padrão. Toda superfície flutuante usa `dark-blue-light` em opacidade parcial (35% para containers de feature, 70% para login, 95% para modais) com `backdrop-blur`, deixando o fundo fotográfico sempre perceptível por trás.

## 3. Typography

**Display/Body Font:** Segoe UI (com fallback Tahoma, Geneva, Verdana, sans-serif)
**Mono Font:** Consolas (com fallback ui-monospace, monospace) — exclusivo para saída de log/terminal

**Character:** Uma única família sem serifa, nativa do Windows — reforça que esta é uma ferramenta de sistema, não um produto de consumo. A variação de peso (600/700 para títulos, 400 para corpo) carrega toda a hierarquia; não há jogo de famílias.

### Hierarchy
- **Title** (700, 1.5rem/24px, leading 1.2): título de cada `FeatureContainer`, sempre em laranja queimado.
- **Heading** (600, 1.125rem/18px, leading 1.3): cabeçalhos de modal, seções de sidebar, accordion.
- **Body** (400, 1rem/16px, leading 1.5): texto corrido, mensagens de modal, descrições de feature.
- **Label** (600, 0.75rem/12px, tracking 0.05em, uppercase): rótulos de campo (`EditableField`), microtextos de estado.
- **Mono** (400, 0.875rem/14px, leading 1.5): saída do `LogPanel`, sempre `whitespace-pre-wrap`.

### Named Rules
**A Regra da Fonte Única.** Segoe UI cobre 100% da interface, exceto a saída de log (Consolas). Não introduzir uma segunda família sans para "variar" — a hierarquia vem de peso e tamanho, não de troca de fonte.

## 4. Elevation

O sistema é majoritariamente plano: a profundidade não vem de `box-shadow`, vem do empilhamento de vidro translúcido sobre o fundo fotográfico. Sombra (`shadow-2xl`/`shadow-md`) aparece apenas em duas situações pontuais — o card de login e o modal — como reforço de que aquele elemento está "no topo" da pilha, nunca como recurso decorativo geral.

### Shadow Vocabulary
- **Elevação de Overlay** (`box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5)` — Tailwind `shadow-2xl`): reservada para o card de Login e o corpo do Modal — os dois únicos elementos que interrompem o fluxo principal.
- **Elevação de Componente** (`box-shadow: 0 4px 6px -1px rgba(0,0,0,0.3)` — Tailwind `shadow-md`): usada no Accordion fechado, para separá-lo sutilmente do painel-mãe.

### Named Rules
**A Regra do Plano-por-Padrão.** Superfícies ficam planas em repouso. Sombra só aparece em elementos que literalmente flutuam por cima de outro conteúdo (login, modal) — nunca em cards de listagem ou containers de feature, que usam blur + opacidade, não sombra, para se destacar do fundo.

## 5. Components

### Buttons
- **Shape:** cantos arredondados médios (6px, `rounded-md`).
- **Primary:** fundo laranja queimado (#ed5f0c), texto ardósia profunda (#1A202C), peso 700, padding 10px 20px. Reservado para a única ação principal de cada tela (Entrar, Confirmar).
- **Secondary:** fundo ardósia profunda (#1A202C), borda 1px cinza-600, texto branco-gelo. Usado para Cancelar/Fechar.
- **Hover / Focus:** primary clareia via `brightness-110`; secondary escurece para `gray-700`. Transição `200ms ease`. Foco visível com `outline` branco de 2px e offset — nunca removido.

### Cards / Containers (FeatureContainer)
- **Corner Style:** 12px (`rounded-xl`).
- **Background:** ardósia clara a 35% de opacidade + `backdrop-blur-md`, deixando o fundo fotográfico visível por trás.
- **Shadow Strategy:** nenhuma — a separação do fundo vem do blur, não de sombra (ver Elevação).
- **Border:** 1px cinza-700, sutil, apenas para definir a borda do vidro.
- **Internal Padding:** 24px, com título em laranja queimado fixo no topo e conteúdo em coluna flexível abaixo.

### Modal
- **Corner Style:** 12px (`rounded-xl`); overlay de fundo em preto 60% + `backdrop-blur-sm`.
- **Background:** ardósia clara a 95% de opacidade — a mais opaca do sistema, porque o modal precisa de legibilidade máxima por interromper o fluxo.
- **Header colorido por tipo:** sucesso (verde), erro (vermelho), aviso (amarelo) — cor aplicada ao ícone e à faixa superior, nunca ao corpo do texto.
- **Entrada:** fade (0.2s) + slide vertical de -20px (0.3s, ease-out). Sem bounce.

### Inputs / Campos Editáveis (EditableField)
- **Style:** fundo ardósia profunda, sem borda lateral colorida. **Divergência deliberada do legado:** o legado usava uma faixa lateral de 4px em roxo estrutural como destaque (`border-l-4`) — um anti-padrão de "side-stripe border". Na versão React, o campo editável usa borda completa sutil (1px, cinza-700) e o estado "editável" é comunicado por um ícone de lápis com hover em laranja queimado, não por uma faixa colorida.
- **Focus:** borda completa muda para roxo estrutural (#171958af) ao entrar em modo de edição; nunca glow ou faixa lateral.
- **Confirmação/Cancelamento:** ícones inline (check verde / X vermelho) ao lado do campo em edição, com `Enter`/`Escape` como atalhos de teclado.

### Navigation (Sidebar)
- **Style:** painel translúcido com `backdrop-blur`, deslizando da direita (`translateX(100%) → 0`, 0.3s ease-out). Título "Módulos" em laranja queimado; cada grupo de módulo tem cabeçalho sublinhado em roxo estrutural.
- **Estados:** itens de navegação ficam desabilitados (opacidade reduzida, sem hover) quando o usuário não está autenticado — a sidebar nunca esconde a existência dos módulos, apenas bloqueia o acesso.
- **Hover/Active:** item ativo recebe leve realce de fundo; hover em laranja queimado apenas no texto, nunca preenchimento total.

### Log Panel (Componente assinatura)
- Painel de saída tipo terminal: fundo ardósia profunda a 50% de opacidade, fonte mono, `whitespace-pre-wrap`, autoscroll para a última linha. É o componente que mais reforça a metáfora do HUD — é onde o técnico vê, em tempo real, exatamente o que a ferramenta está fazendo na máquina.

## 6. Do's and Don'ts

### Do:
- **Do** usar laranja queimado (#ed5f0c) exclusivamente para a ação primária de cada tela — um único ponto de destaque por vez.
- **Do** manter toda superfície de conteúdo translúcida com `backdrop-blur` sobre o fundo fotográfico; a profundidade vem dessa camada, não de sombra.
- **Do** dar feedback explícito (log, modal, ou estado de botão) para toda ação — nunca deixar o técnico sem confirmação do que aconteceu.
- **Do** usar exclusivamente Segoe UI (corpo/título) e Consolas (log); peso e tamanho carregam a hierarquia, não troca de família.
- **Do** aplicar sombra (`shadow-2xl`) apenas em login e modal — os únicos elementos que literalmente flutuam sobre o resto da interface.

### Don't:
- **Don't** usar faixas laterais coloridas (`border-l-4` ou similar) como destaque visual — isso é o anti-padrão "side-stripe border"; o legado fazia isso no EditableField e a migração corrige isso com borda completa + ícone.
- **Don't** usar texto em gradiente, eyebrows numerados decorativos ou glassmorphism sem propósito funcional — são clichês de "IA generativa" explicitamente rejeitados em PRODUCT.md.
- **Don't** adotar estética de terminal hacker/cyberpunk (verde sobre preto, monoespaçada agressiva) — mesmo em telas que tocam registro e firewall, o tom é de instrumento de precisão, não de ferramenta de invasão.
- **Don't** criar grades de cards idênticos ou painéis administrativos genéricos estilo CRUD — cada feature tem sua própria composição dentro do `FeatureContainer`, não um template repetido sem distinção.
- **Don't** introduzir uma segunda cor de acento — se algo "precisa" de destaque além do laranja, repensar a hierarquia, não adicionar cor.
