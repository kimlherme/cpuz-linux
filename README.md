# CPUZ Linux

Benchmark de CPU em Rust para Linux, com interface no terminal, testes single e multi e identificação do processador em uso.

**Experimental:** a carga foi reconstruída a partir da investigação do 17.01.SSE x64; equivalência numérica e de pontuação com o CPU-Z ainda não foi demonstrada. Este projeto não é afiliado à CPUID.

![Prévia ilustrativa da interface no terminal](docs/tui-preview.svg)

## Começar

Com Rust estável recente, Git e compilador/linker instalados:

```bash
git clone https://github.com/kimlherme/cpuz-linux.git
cd cpuz-linux
cargo build --release --locked
./target/release/cpuz-linux
```

Sem argumentos, em um terminal interativo, o programa abre a TUI. Pressione **S** para single, **M** para multi ou **B** para executar ambos. O nome da CPU, arquitetura, CPUs disponíveis e núcleos físicos detectados aparecem no painel superior.

Use release para medir desempenho. O build debug tem características de otimização diferentes e não serve como comparação de score com release.

## Interface

A TUI mostra cartões de single e multi, tempo decorrido, quantidade de trabalho, número de workers e razão multi/single. As configurações podem ser ajustadas enquanto não houver medição ativa.

| Tecla | Ação |
| --- | --- |
| `S` | Executar single com uma thread |
| `M` | Executar multi com o número de workers selecionado |
| `B` | Executar multi e depois single |
| `+` / `-` | Aumentar / reduzir workers do multi |
| `[` / `]` | Reduzir / aumentar duração por fase |
| `C` | Cancelar a medição atual; resultado fica identificado como parcial |
| `Q` / `Esc` | Sair |
| `Ctrl+C` | Cancelar e sair |

O terminal deve ter pelo menos 64 colunas e 22 linhas para mostrar o painel completo. Em tamanhos menores, a aplicação pede para ampliar a janela. As threads em andamento são finalizadas antes da restauração do terminal; cancelamento pode esperar o término da unidade atual.

A imagem acima é uma prévia ilustrativa do layout. Ela não representa uma medição de uma CPU de referência.

## Usar sem interface

```bash
# Single: uma thread, dez segundos
cpuz-linux --mode single --seconds 10 --no-tui

# Multi: CPUs disponíveis ao processo
cpuz-linux --mode multi --threads auto --seconds 10

# Multi com oito workers, depois single com uma thread
cpuz-linux --mode both --threads 8 --seconds 10

# JSON para guardar e analisar resultados
cpuz-linux --mode both --threads auto --seconds 10 --json > resultado.json

# Abrir TUI com configurações iniciais
cpuz-linux --tui --threads 4 --seconds 15

# Ajuda
cpuz-linux --help
```

Quando `cpuz-linux` ainda não estiver no PATH, use `./target/release/cpuz-linux` nos exemplos.

### Opções

| Opção | Valores | Padrão |
| --- | --- | --- |
| `--mode` | `single`, `multi`, `both` | `single` |
| `--seconds` | Número entre `0.05` e `3600` | `10` por fase |
| `--threads` | `auto` ou inteiro entre `1` e `256` | CPUs disponíveis ao processo, limitado a 256 |
| `--tui` | Abre interface interativa | Ativada sem argumentos quando stdin/stdout são terminais |
| `--no-tui` | Saída de texto | Ativada fora da abertura automática da TUI |
| `--json` | Objeto JSON, sem TUI | Desativado |
| `-h`, `--help` | Mostra ajuda | — |

`--threads` configura o multi; single sempre usa um worker. `both` executa duas fases: multi e depois single. Com `--seconds 10`, são aproximadamente vinte segundos de medição, além da preparação e finalização.

Sem argumentos e sem terminal, o programa executa single por dez segundos. `--tui` precisa de stdin e stdout ligados a um terminal e não pode ser combinado com `--json` ou `--no-tui`. Valores inválidos produzem mensagem de erro e código de saída 2.

A TUI aguarda uma tecla de execução. `--mode` não dispara automaticamente um teste ao abrir a interface; as teclas S/M/B escolhem o que executar.

## Instalação nas distribuições

O projeto usa Rust edition 2024 e exige Rust 1.88 ou posterior. As dependências da aplicação são Rust; a compilação requer também ferramentas nativas para linkar o binário. Os caminhos abaixo atendem Arch Linux, Ubuntu, Fedora e openSUSE. Consulte [distribuições](docs/distributions.md) para empacotamento e detalhes.

### Arch Linux

```bash
sudo pacman -S --needed base-devel git curl rustup
rustup default stable
```

### Ubuntu

```bash
sudo apt update
sudo apt install build-essential git curl ca-certificates
```

Instale um Rust estável recente pelo procedimento universal abaixo se a versão disponível no sistema for anterior à 1.88. A versão de Rust dos repositórios depende da versão do Ubuntu.

### Fedora

```bash
sudo dnf install gcc git curl
```

Use um toolchain Rust recente do sistema ou o procedimento universal abaixo.

### openSUSE

```bash
sudo zypper install gcc git curl
```

Use um toolchain Rust recente do sistema ou o procedimento universal abaixo. A disponibilidade e versão de pacotes Rust variam entre Leap e Tumbleweed.

### Rust estável com rustup

O instalador oficial está em [rustup.rs](https://rustup.rs/). Para baixar o script, examiná-lo e então executá-lo:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/cpuz-rustup-init.sh
less /tmp/cpuz-rustup-init.sh
sh /tmp/cpuz-rustup-init.sh --profile minimal --default-toolchain stable
. "$HOME/.cargo/env"
rustc --version
```

Se já usa rustup, atualize seu toolchain estável com `rustup update stable`. O arquivo `rust-toolchain.toml` seleciona o canal utilizado pelo projeto.

### Instalar no usuário

Depois de clonar o repositório e entrar na pasta:

```bash
./scripts/install.sh --prefix "$HOME/.local"
```

O instalador compila com `--release --locked` e instala o executável, documentação e manual no prefixo selecionado. Garanta que `$HOME/.local/bin` esteja no PATH; depois execute `cpuz-linux`.

Para remover os arquivos instalados nesse prefixo:

```bash
./scripts/install.sh --prefix "$HOME/.local" --uninstall
```

Os scripts e receitas de pacote estão no repositório. Ter um caminho de instalação documentado não significa que cada versão de cada distribuição tenha sido executada manualmente; consulte os resultados concretos de CI e o estado de validação.

## O que o score mede

A unidade recuperada para o single 17.01 x64 é **pixels contabilizados por milissegundo**. Cada pixel envolve 34 avaliações de ruído 2D, além da composição de cores.

```text
score = float32(pixels contabilizados) / float32(tempo medido em ms inteiros)
```

500 pontos correspondem aproximadamente a 500 mil pixels por segundo dessa carga. O cálculo é na CPU; os pontos não são frames da GPU nem GHz. No caminho absoluto da interface original inspecionado, a pontuação é mostrada com uma casa decimal, sem fator de calibração por uma CPU-base.

O multi executa a carga de verdade em várias threads e soma seu trabalho aceito. A razão multi/single é o score multi dividido pelo single. Ela depende também de turbo, temperatura, SMT e limites de energia.

Cada worker utiliza um buffer privado de 4 MiB, além das demais estruturas. Muitos workers aumentam consumo de memória e competição por recursos; 256 workers representam aproximadamente 1 GiB apenas em buffers. O limite aceito pela interface não é uma recomendação de desempenho.

## JSON e automação

`--json` escreve um objeto com `schema_version: 1`. Os campos principais são:

| Campo | Conteúdo |
| --- | --- |
| `algorithm` | Identificador da carga implementada |
| `experimental` | Estado experimental |
| `cpu` | Modelo, arquitetura e contagem de CPUs |
| `configuration` | Configuração interpretada da execução |
| `results` | Uma entrada por fase executada |
| `multi_single_ratio` | Razão quando há single e multi válidos; `null` caso contrário |

Cada resultado contém `mode`, `threads`, `pixels`, `elapsed_ms`, `score`, `checksum`, `worker_pixels` e `cancelled`. `worker_pixels` permite conferir a soma do trabalho entre workers. O checksum resume o estado final dos buffers; como a quantidade e a posição de trabalho dependem do tempo, ele não é um vetor fixo de equivalência com o original.

Para comparações, preserve os resultados completos e registre commit, compilador, perfil de energia, afinidade e temperatura. Execute várias vezes e compare a dispersão, além da mediana. O par informado pelo usuário foi 523 na CLI Linux e 535 no CPU-Z Windows, cerca de 2,24% de diferença; esse relato isolado não estabelece equivalência.

## Documentação da investigação

- [Investigação completa do 17.01 x64](docs/reverse-engineering.md): artefatos e hashes, extração, classes, ruído simplex, gradientes, campos, pixels, threads, score, interface e exportação.
- [Índice de evidências](docs/evidence-index.md): Evidence IDs, operações, endereços e limites de cada conclusão.
- [Metodologia de medição e validação](docs/methodology.md): frequência, turbo, núcleos híbridos, repetições e comparação Windows/Linux.
- [Arquitetura da implementação](docs/architecture.md): componentes Rust, organização do multi, tempo, contagem, checksum e interface.
- [Distribuições e instalação](docs/distributions.md): Arch, Ubuntu, Fedora e openSUSE.
- [Validação executada](docs/validation.md): testes, TUI em pseudo-terminal e resultados nas quatro distribuições.

A análise cobriu em detalhe o 17.01.SSE x64. As variantes 19.01.SSE, 19.01.AVX2 e 19.01.AVX512 foram identificadas no executável, mas não são implementações anunciadas por esta CLI. A tabela numérica, os pixels de referência e a política completa de encerramento do original ainda precisam de validação.

O histórico de engenharia reversa e a implementação informada por ele estão descritos de forma explícita. Este projeto não afirma ter completado um processo clean room com isolamento verificável do implementador.

## Licença

O código deste repositório é distribuído sob [GPL-2.0-only](LICENSE). CPU-Z e CPUID são nomes de seus respectivos titulares. A licença do projeto não concede direitos sobre os executáveis proprietários analisados, que não são distribuídos aqui.
