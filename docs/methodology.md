# Medição, comparação e validação

O score deste projeto é experimental. A proximidade entre dois números, por si só, não estabelece que os programas executaram a mesma carga. A comparação deve passar por três etapas: identidade da carga numérica, identidade da contabilidade e comparação de desempenho.

Este documento define um procedimento reproduzível e registra as observações disponíveis. Ele não apresenta resultados ainda não coletados como testes concluídos.

## Resultados informados pelo usuário

Na conversa de desenvolvimento, o usuário informou os seguintes resultados:

| Programa e sistema | Score single informado | Origem |
| --- | ---: | --- |
| CLI Rust no Linux | 523 | Relato do usuário |
| CPU-Z no Windows | 535 | Relato posterior do usuário, esclarecido como CPU-Z |

A diferença relativa usando o CPU-Z como denominador é:

```text
(535 - 523) / 535 × 100 = 2,242990654… %
```

A CLI ficou aproximadamente 2,24% abaixo nesse par de valores. Não foram anexados logs de execução, série de repetições, versão do compilador, dados térmicos ou confirmação de frequência efetiva para esse par. Esses números não foram medidos novamente pelos autores desta documentação.

O usuário também relatou que uma execução anterior ocorreu com limite de 3 GHz, explicando um resultado da ordem de 300. Esse limite é contexto informado pelo usuário; não há registro de telemetria que permita reconstruir a frequência exata durante aquela execução.

O resultado CPU-Z inicialmente mencionado como 549 foi substituído, na comparação esclarecida, por 535. Não se deve escolher um dos valores para obter uma diferença mais favorável. É necessário preservar cada amostra com suas condições quando houver coleta formal.

## Por que 2,24% não comprova equivalência

Um resultado próximo pode decorrer de diferenças que se compensam. Por exemplo, um núcleo de ruído mais barato pode ser acompanhado por um despachante mais caro. Isso produziria números próximos sem representar a mesma carga.

Saídas numericamente idênticas também não obrigam tempos idênticos: instruções geradas, inlining, layout de memória, sincronização e bibliotecas podem mudar a velocidade. A meta deve declarar se busca equivalência funcional, comparabilidade de score ou ambas.

| Critério | Evidência necessária |
| --- | --- |
| Mesma unidade | Contador de pixels e tempo em ms explicitados |
| Mesma função matemática | Valores intermediários e pixels do original |
| Mesma política de medição | Instantes de início/fim, trabalho aceito e unidade de trabalho |
| Scores comparáveis | Séries controladas no mesmo hardware, com dispersão |
| Mesmo desempenho universal | Não pode ser inferido de uma única CPU |

## Condições a registrar

Para cada série, guarde um arquivo de resultados e uma ficha com as condições. O JSON da CLI é adequado para preservar a saída estruturada; informações externas ao processo precisam ser registradas separadamente.

| Categoria | Informações úteis |
| --- | --- |
| Hardware | Modelo completo da CPU, número de núcleos e threads, RAM |
| Topologia | SMT ligado/desligado, CPUs lógicas permitidas, P-cores/E-cores |
| Firmware | Versão de BIOS/UEFI, configurações relevantes de energia |
| Sistema | Distribuição, kernel, versão Windows quando aplicável |
| Energia | Fonte ou bateria, perfil de energia, governor, limite de potência |
| Frequência | Limite configurado e frequência efetiva durante a carga |
| Temperatura | Estado inicial, pico, indícios de throttling |
| Build | Commit, rustc, target, flags, release/debug |
| Execução | Modo, duração, threads, afinidade, TUI ou modo texto |
| Contabilidade | Pixels aceitos, elapsed real, score calculado |
| Ambiente | VM, contêiner, cgroups, cpuset, outros processos relevantes |

O nome da CPU mostrado na TUI identifica o processador reportado pelo sistema. Ele não comprova a frequência, o núcleo que executou cada unidade ou ausência de limites de energia.

## Frequência, turbo e núcleos híbridos

Um limite de 3 GHz muda diretamente a capacidade de executar a carga por segundo. Comparar uma amostra limitada com outra em turbo mistura duas condições. A frequência configurada também não é necessariamente a frequência efetiva: limites térmicos e de potência podem reduzi-la.

Em CPUs híbridas, uma thread pode cair num P-core ou E-core. Uma migração durante o teste pode mudar o score. Uma série single deve usar a mesma classe de núcleo em todas as repetições, idealmente com afinidade registrada.

SMT compartilha recursos entre threads do mesmo núcleo físico. O total de CPUs lógicas não equivale a tantos núcleos físicos independentes. No multi, mais threads podem aumentar throughput sem multiplicá-lo linearmente; também podem reduzir a frequência de cada núcleo.

A razão multi/single descreve o ganho entre as duas execuções coletadas. Não é uma medida isolada de eficiência de SMT ou escalabilidade porque também incorpora turbo, energia, temperatura e políticas de agendamento.

## Procedimento recomendado para uma série

1. Compile em release a partir de um commit conhecido e registre a versão do Rust.
2. Confirme que o mesmo algoritmo e a mesma duração são usados em ambos os sistemas.
3. Registre perfil de energia, limite de frequência, topologia e afinidade.
4. Aguarde uma condição térmica estável; escolha e registre uma política de aquecimento.
5. Execute pelo menos cinco repetições de cada configuração.
6. Preserve todas as amostras, inclusive resultados baixos ou altos.
7. Calcule mediana, mínimo, máximo e uma medida de dispersão.
8. Repita a configuração inicial ao final para detectar deriva térmica ou de frequência.
9. Ao comparar sistemas, use o mesmo hardware e registre a ordem das séries.
10. Interprete a diferença junto à dispersão, sem transformá-la automaticamente em erro de implementação.

Cinco repetições são uma escolha inicial prática, não uma garantia estatística. Se a dispersão for semelhante à diferença entre programas, serão necessárias mais amostras ou melhores condições de controle.

### Mediana e deriva

A mediana reduz a influência de uma interrupção isolada, mas não corrige um sistema que aquece progressivamente. Uma sequência 540, 535, 530, 525, 520 indica deriva; reduzir tudo a 530 esconderia essa tendência.

Uma comparação útil apresenta a série inteira ou, pelo menos, mediana e intervalo. A média pode ser acrescentada, com o número de amostras e método de exclusão explícitos. Não remova outliers apenas porque afastam o resultado desejado.

## Afinidade e limites do processo

Uma afinidade externa permite selecionar CPUs lógicas para a CLI no Linux. Exemplo, após confirmar que a CPU lógica 2 existe e é permitida:

```bash
taskset -c 2 cpuz-linux --mode single --seconds 10 --no-tui
```

Para um subconjunto em multi:

```bash
taskset -c 0-3 cpuz-linux --mode multi --threads 4 --seconds 10 --json
```

Esses comandos são exemplos de metodologia, não recomendações de que as CPUs 0–3 sejam equivalentes em toda máquina. Em um processador híbrido, consulte a topologia antes de escolher o conjunto.

Cpuset e afinidade restringem onde o processo pode executar. Cgroups também podem limitar o tempo total de CPU por período, mesmo quando muitas CPUs lógicas estão visíveis. Uma quota de duas CPUs em um contêiner não libera o desempenho de dezesseis CPUs só porque o programa criou dezesseis threads.

Virtualização adiciona outra camada de agendamento. Compare resultados em VM com essa condição identificada, sem tratá-los automaticamente como desempenho nativo.

## Tempo e contagem

A reconstrução precisa distinguir duração solicitada, tempo decorrido e trabalho aceito. A fórmula recuperada para o single original usa tempo medido com conversão inteira para ms.

Uma unidade de 1024 pixels só deve aumentar o numerador quando satisfizer a política de conclusão da implementação. Contar pixels antes do cálculo, ou somar uma unidade incompleta após cancelamento, superestima o trabalho.

No multi, todos os workers devem compartilhar uma janela coerente. Iniciar o relógio muito antes de criar threads inclui setup; iniciar cada worker com uma janela própria e dividir pela duração de um único worker combina intervalos diferentes. A arquitetura deve declarar a decisão adotada.

O tempo de teardown merece tratamento explícito. Esperar um worker terminar garante liberação de recursos, mas não significa que todo trabalho terminado durante essa espera deve entrar no score. A investigação do original não encontrou coleta adicional no epílogo single seguido.

Para janelas muito curtas, milissegundos inteiros têm peso relativo maior. Com 50 ms, 1 ms representa 2%; com 10000 ms, representa 0,01%. Janelas curtas servem para exploração da interface e diagnóstico, e têm menor utilidade para comparações finas.

## Diferenças numéricas que precisam ser auditadas

### Tabela e seno

O original aloca uma tabela de 4096 floats, mas a primeira investigação não estabeleceu completamente a auxiliar de preenchimento. Se a reconstrução usa seno, o resultado pode depender da biblioteca, da precisão intermediária e de quando ocorre a conversão para float32.

Mesmo um erro pequeno na tabela pode ser amplificado pela multiplicação por 43758.547 e pela extração da parte fracionária. Isso pode alterar gradientes, campos e pixels. Um checksum diferente é relevante mesmo quando o score continua próximo.

### Piso, truncagem e negativos

O índice da célula simplex usa piso; a extração de frações em outra etapa pode usar truncagem. Para -0.2, truncagem e piso retornam respectivamente 0 e -1. Tratar todas as etapas com a mesma função modifica o algoritmo.

### Saturação de canais

A ordem entre limitar valores, multiplicar por 255 e converter para inteiro importa. A semântica de um cast Rust saturante não deve ser assumida igual à conversão x86 para todos os valores possíveis. Os testes devem cobrir as fronteiras normais da carga e declarar o tratamento de entradas não finitas.

### FMA, precisão e otimização

A soma de um produto com outro termo pode arredondar de forma diferente quando fundida. Float64 intermediário também muda o resultado em relação a float32 por etapa. Uma alteração de flags deve vir acompanhada de comparação de saídas, além da medição de velocidade.

Não introduza fator de calibração para fazer um processador específico chegar a 535. Isso esconderia diferenças da carga e prejudicaria a interpretação em outras CPUs. A escala recuperada é trabalho por tempo; uma aproximação numérica deve ser corrigida na função ou identificada como aproximação.

## Plano de validação em camadas

### 1. Identidade e build

Preserve hash do original, versão do benchmark, commit da reconstrução e parâmetros do compilador. A análise disponível cobre o 17.01.SSE x64 do executável identificado em [investigação](reverse-engineering.md), não qualquer versão do CPU-Z encontrada online.

### 2. Vetores de tabela e gradientes

Colete valores de referência da inicialização e do gerador de gradientes. Inclua coordenadas positivas, negativas, zero e fronteiras de indexação. Compare padrões float32 quando a meta for identidade de bits.

### 3. Núcleo simplex

Compare a seleção do triângulo, os três gradientes, os pesos e a soma final. Inclua igualdade entre coordenadas locais e pontos próximos das fronteiras. Um diagnóstico que mostre apenas a cor final perde a localização do primeiro desvio.

### 4. Campos e pixels

Compare os acumuladores das quatro sequências, a combinação dos campos, os canais anteriores à conversão e os bytes finais. Compare uma linha completa e, depois, a grade inteira.

Checksums de grades são úteis para detectar alterações. Um checksum produzido somente pela reconstrução comprova determinismo interno daquela versão; não comprova equivalência com o original sem uma referência correspondente.

### 5. Contabilidade

Use um modo de trabalho fixo para confirmar que N unidades produzem exatamente N × 1024 pixels aceitos. Depois examine deadline, cancelamento, conclusão e coleta do último worker. Contagem matemática e medição temporal devem ser avaliadas separadamente.

### 6. Windows nativo e Linux

Compare o original em Windows nativo e a reconstrução, preferencialmente também compilada para Windows quando essa plataforma estiver implementada. Uma execução da reconstrução no mesmo Windows ajuda a separar parte do efeito do sistema operacional do efeito do kernel e compilador.

Wine é útil para exploração, mas adiciona outra camada. Um resultado em Wine não substitui a referência de Windows nativo e não estabelece a mesma política de threads, prioridade e timer.

### 7. Série de desempenho

Somente após verificar a carga, faça a série repetida no mesmo hardware. Apresente dispersão, condições térmicas e limites. Em múltiplas CPUs, verifique se o erro relativo muda com arquitetura, classe de núcleo e número de threads.

## Estado atual da evidência

| Item | Estado documental |
| --- | --- |
| Fórmula do score single original | Recuperada estaticamente |
| Caminho até UI/export específico | Recuperado estaticamente |
| Relatos 523 e 535 | Registrados como dados do usuário |
| Vetores de referência do original | Não disponibilizados nesta investigação |
| Equivalência de tabela/pixels | Não demonstrada |
| Série controlada repetida Windows/Linux | Não disponibilizada |
| Scores universalmente comparáveis | Não demonstrado |

Resultados de build e testes da versão atual devem estar no README ou na saída de CI correspondente. Este documento é um plano de medição e uma descrição dos limites das evidências disponíveis, e não uma declaração de que todas essas etapas já ocorreram.
