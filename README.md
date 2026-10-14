# CPU-Z-like benchmark para Linux

Benchmark de linha de comando, escrito em C++17, que reproduz parte do formato
de carga do teste de CPU do CPU-Z 19.01. O programa mede quantos pixels consegue
processar por segundo em uma imagem gerada por software.

O projeto é uma implementação independente e escalar. Os detalhes completos do
algoritmo original não estão disponíveis nesta reconstrução; portanto, o
resultado serve para experimentar e comparar execuções deste programa, mas não
é uma pontuação oficial do CPU-Z nem deve ser comparado diretamente com a
pontuação exibida pelo CPU-Z.

## Conteúdo

- [Requisitos](#requisitos)
- [Compilar](#compilar)
- [Executar](#executar)
- [Interface interativa (TUI)](#interface-interativa-tui)
- [Opções](#opções)
- [Entender o resultado](#entender-o-resultado)
- [Como o teste funciona](#como-o-teste-funciona)
- [Limitações e comparação de resultados](#limitações-e-comparação-de-resultados)
- [Solução de problemas](#solução-de-problemas)

## Requisitos

- Linux
- Compilador C++ com suporte a C++17, como GCC ou Clang
- `make` (opcional; também é possível compilar com um comando direto)
- Suporte a threads POSIX, normalmente fornecido pela biblioteca padrão do Linux

Em Debian e Ubuntu, instale as ferramentas de compilação com:

```sh
sudo apt update
sudo apt install build-essential
```

## Compilar

Na raiz do repositório:

```sh
make
```

O executável será criado como `cpu-z-bench-linux`. Para compilar sem `make`:

```sh
g++ -O3 -std=c++17 -Wall -Wextra -Wpedantic -ffp-contract=off -pthread \
  cpu-z-bench-linux.cpp -o cpu-z-bench-linux
```

Para apagar o executável gerado:

```sh
make clean
```

## Executar

Sem argumentos, o programa executa um teste single-thread de 10 segundos:

```sh
./cpu-z-bench-linux
```

Exemplos com duração e modo explícitos:

```sh
# Single-thread por 10 segundos
./cpu-z-bench-linux --mode single --duration-ms 10000

# Multi-thread usando os CPUs permitidos para este processo
./cpu-z-bench-linux --mode multi --duration-ms 10000 --threads auto

# Multi-thread usando 4 workers
./cpu-z-bench-linux --mode multi --duration-ms 10000 --threads 4

# Executa multi-thread e depois single-thread, ambos por 5 segundos
./cpu-z-bench-linux --mode both --duration-ms 5000 --threads 4
```

Consulte a ajuda embutida com:

```sh
./cpu-z-bench-linux --help
```

## Interface interativa (TUI)

Para escolher as opções e iniciar execuções por um menu no terminal:

```sh
./cpu-z-bench-linux --tui
```

No menu, `m` alterna entre os modos single, multi e both; `d` define a duração
em milissegundos; `w` define o número de workers ou `auto`; `r` inicia a
execução; `h` mostra os controles e `q` sai. Depois de cada execução, o menu
volta a aparecer. A duração medida se aplica separadamente a cada modo
selecionado; cada execução faz também 150 ms de aquecimento antes de iniciar o
cronômetro.

Também é possível abrir o menu com opções iniciais, por exemplo:

```sh
./cpu-z-bench-linux --tui --mode both --duration-ms 3000 --threads auto
```

A interface usa apenas entrada e saída padrão do terminal, sem dependências
extras, e requer uma sessão interativa.

## Opções

| Opção | Valores | Padrão | Descrição |
|---|---|---|---|
| `--mode` | `single`, `multi` ou `both` | `single` | Escolhe o teste single-thread, multi-thread ou os dois. |
| `--duration-ms` | De `1` a `86400000` | `10000` | Duração aproximada de cada execução, em milissegundos. |
| `--threads` | `auto` ou de `1` a `1024` | `auto` | Número de workers no modo multi-thread. Ignorado no modo `single`. |
| `--tui` | — | desativado | Abre o menu interativo do terminal. |
| `--help`, `-h` | — | — | Mostra as opções disponíveis. |

### Modos

- `single`: processa os frames com uma thread. É o modo padrão e o mais útil
  para observar o desempenho de um único núcleo lógico.
- `multi`: divide as linhas da imagem entre os workers e mede o processamento
  paralelo. Este modo não executa também o teste single-thread.
- `both`: executa primeiro o teste multi-thread, espera 10 ms e executa o
  single-thread. A duração informada se aplica separadamente a cada execução.

No modo multi-thread, `--threads auto` usa o número de CPUs na máscara de
afinidade do processo, quando o Linux fornece essa informação. Se a consulta de
afinidade não estiver disponível, o programa usa a quantidade informada pela
biblioteca padrão. O valor `auto` não detecta limites de quota de CPU que não
estejam refletidos na afinidade.

## Entender o resultado

Cada execução apresenta o tempo decorrido, a quantidade de pixels completos
processados, a taxa de pixels por segundo e uma pontuação na escala aproximada
do CPU-Z 19.01:

```text
Single-thread:
  CPU: Intel(R) ...
  reported clock (/proc/cpuinfo): 3400.0-4100.0 MHz
  elapsed: ... s
  pixels: ...
  pixels/s: ...
  19.01-scale score (pixels/s / 10000): ...
```

As métricas são calculadas assim:

```text
pixels/s = pixels processados / tempo decorrido em segundos
pontuação da escala = pixels/s / 10.000
```

A contagem considera cada posição da imagem uma vez. Ela não multiplica o total
pelas quatro componentes RGBA, pelas camadas de ruído ou pelo número de
operações internas.

Os números variam conforme o processador, frequência, temperatura, carga do
sistema, limites da máquina virtual ou do contêiner e duração selecionada. A
pontuação com uma casa decimal pode arredondar pequenas diferenças; consulte
também `pixels/s` ao comparar execuções.

O modelo vem do campo `model name` de `/proc/cpuinfo`. A frequência mostra o
intervalo dos valores `cpu MHz` disponíveis nesse arquivo, lidos após cada
execução; ela é uma amostra informativa e pode variar entre núcleos ou não estar
disponível em máquinas virtuais.

## Como o teste funciona

Antes de cada execução medida, o programa faz 150 ms de aquecimento com o mesmo
modo e número de workers selecionados; esse trabalho não entra na contagem nem
no tempo reportados. O aquecimento ajuda o processador a sair do estado idle,
especialmente em testes curtos.

O programa cria uma tabela determinística com 4096 valores antes de iniciar a
medição. Em seguida, processa imagens de 1024 × 1024 pixels. Para cada pixel,
calcula duas composições de ruído 2D no estilo simplex: uma com 10 camadas e
outra com 7. Os resultados são convertidos em quatro componentes de cor (RGBA)
e gravados em um buffer.

No modo single-thread, uma thread processa a imagem inteira. No modo
multi-thread, os workers recebem faixas separadas de linhas e sincronizam entre
frames. A preparação das threads e dos buffers ocorre antes do intervalo
cronometrado.

O programa interrompe o trabalho quando termina o tempo solicitado e contabiliza
somente pixels de linhas que foram concluídas. Por isso, a duração real pode
variar ligeiramente em torno do valor configurado: a verificação do tempo
acontece durante o processamento das linhas.

## Limitações e comparação de resultados

Esta versão foi reconstruída a partir de uma análise estática do caminho de
teste do CPU-Z 19.01. A análise não recuperou todos os detalhes da tabela,
hash, fórmulas das composições e conversão de cor; esses pontos foram
aproximados. Além disso, esta implementação usa um kernel escalar e não contém
os caminhos SIMD SSE, AVX2 ou AVX-512 do CPU-Z.

Por essas diferenças, a etiqueta `19.01-scale score` indica somente a escala
usada para converter o throughput. Ela não torna os resultados equivalentes à
pontuação oficial. Uma pontuação como `163.8` significa que esta implementação
mediu aproximadamente 1.638.000 pixels/s; não significa que um CPU-Z oficial
tenha pontuado 163,8 no mesmo processador.

Para comparar mudanças neste programa, use a mesma máquina, o mesmo modo, a
mesma duração e a mesma quantidade de threads. Sempre que possível, execute
mais de uma vez em condições semelhantes e compare `pixels/s`. Feche tarefas
pesadas em segundo plano e registre se o teste foi executado em hardware físico,
máquina virtual ou contêiner. Em ambientes virtualizados, os CPUs visíveis e as
quotas impostas pelo host podem limitar o resultado.

## Solução de problemas

### `g++: command not found` ou `make: command not found`

Instale o compilador e as ferramentas de compilação da sua distribuição. Em
Debian ou Ubuntu, use `sudo apt install build-essential`.

### `Unknown option` ou mensagem de valor inválido

Use `--help` para consultar a sintaxe. `--duration-ms` aceita valores de 1 a
86400000; `--threads` aceita `auto` ou valores de 1 a 1024.

### Resultado diferente entre execuções

Variações podem ocorrer por mudanças de frequência e temperatura do processador,
carga do sistema, afinidade, quota da máquina virtual ou contenção de recursos.
Repita o teste sob condições semelhantes antes de interpretar diferenças
pequenas.

## Estado do projeto

Este é um benchmark experimental e independente para Linux. Não é uma
implementação oficial nem é afiliado à CPUID, fabricante do CPU-Z. Este projeto
está licenciado sob a GNU General Public License versão 2.0; consulte o arquivo
[LICENSE](LICENSE).
