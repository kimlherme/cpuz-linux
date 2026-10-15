# Validação executada da versão 0.2.0

Sessão de 15 de outubro de 2026. A validação comprova funcionamento desta
implementação; não comprova equivalência de pixels, instruções ou score com
o executável Windows do CPU-Z.

## Host Linux x86_64

Compilador: `rustc 1.99.0 (b940084d7 2026-09-28)`.
Modelo detectado: `12th Gen Intel(R) Core(TM) i5-1235U`.

| Verificação executada | Resultado |
| --- | --- |
| `cargo fmt --check` | Passou |
| `cargo test --locked --offline` | 9 testes passaram |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | Passou |
| `cargo build --release --locked --offline` | Passou |
| `python3 scripts/test-tui.py` | Passou |
| `sh -n scripts/install.sh scripts/package.sh` | Passou |
| `bash -n packaging/arch/PKGBUILD` | Passou |
| Instalar em prefixo temporário e executar single/multi JSON | Passou |
| Remover a instalação temporária | Passou |

Os testes de Cargo cobrem parsing/limites, retorno de `--threads auto` após
um valor explícito, detecção da CPU, contagem multi por worker, fórmula com
denominador comum, cancelamento, cálculo na origem, coordenadas negativas,
escrita de linha, saída JSON e recusa da TUI quando executada em pipe.

O teste de UI usa um **pseudo-terminal real**, não somente um buffer de
desenho: abre o programa, confere o painel da CPU, envia B para executar as
duas fases, redimensiona o terminal, inicia/cancela multi, sai com Q e
verifica que os atributos termios foram restaurados. O teste de layout
Ratatui verifica separadamente a renderização e o estado de terminal pequeno.

A prévia SVG foi gerada pelo mesmo caminho de renderização do programa,
com um TestBackend e a CPU detectada. É uma tela em repouso, sem pontuações
inventadas. Para regenerá-la deliberadamente:

```sh
CPUZ_PREVIEW_PATH=docs/tui-preview.svg cargo test --locked layout_shows_cpu_and_survives_small_terminal
```

## Containers de distribuições

Foram usados containers descartáveis com compilador Rust 1.99.0, fontes de
dependências vendorizadas fora do repositório e limite de duas CPUs por
container. Cada ambiente instalou seu GCC/libc nativos e compilou o projeto
do zero. A fonte vendorizada foi verificada pelo Cargo usando o lockfile.
O vendor e caches temporários não são distribuídos neste repositório.

| Ambiente observado | Testes Cargo | TUI em PTY | Single + multi JSON | Instalar/remover | Empacotamento |
| --- | --- | --- | --- | --- | --- |
| Ubuntu 24.04.5 LTS, imagem `ubuntu:24.04` | 9 passaram | Passou | Passou | Passou | DEB gerado e inspecionado |
| Arch Linux, imagem `archlinux:latest` | 9 passaram | Passou | Passou | Passou | Sintaxe do PKGBUILD verificada |
| Fedora 44, imagem `fedora:44` | 9 passaram | Passou | Passou | Passou | RPM gerado e inspecionado |
| openSUSE Tumbleweed 20261013 | 9 passaram | Passou | Passou | Passou | RPM gerado e inspecionado |

O smoke test de cada distro executou `--mode both --seconds 0.2 --threads 2
--json`, verificou dois resultados positivos, uma fase multi com dois
workers e uma single com um. Os testes conferem também que a contagem multi
é a soma do trabalho dos workers.

Após instalar em um prefixo temporário, foram conferidos o executável,
`docs/reverse-engineering.md` e `docs/tui-preview.svg`, preservando links
relativos. A remoção do executável também foi verificada.

O PKGBUILD não foi submetido ao AUR e seu ciclo `makepkg -si` completo não
foi executado nesta sessão. A aplicação foi compilada/executada no Arch,
e o caminho de instalação genérico foi testado. DEB/RPM foram gerados e
consultados com `dpkg-deb --info` / `rpm -qp --info`; a instalação desses
pacotes com o gerenciador de sistema não faz parte desta validação.

## Revisão e publicação

Uma revisão independente de leitura cobriu gate de início, falha de criação
de threads, cancelamento/panic, retorno do timer, soma do score e restauração
da TUI. Corrigimos o tratamento de `--threads auto` repetido e preservamos
a estrutura dos documentos instalados antes da rodada final das quatro
distribuições.

O workflow GitHub Actions reproduz testes similares. A validação local em
containers não implica que uma execução remota de Actions já tenha terminado;
consulte o status do workflow para o commit publicado.

## Limites

- Containers são uma verificação de compatibilidade, não um ambiente ideal
  para comparar desempenho físico de CPUs.
- Não houve execução do CPU-Z original nesta sessão.
- O relato anterior 523/535 foi fornecido pelo usuário e continua separado
  destas verificações; não foi usado para ajustar coeficientes.
- Tabela trigonométrica, pixels intermediários e comportamento do original
  ainda precisam dos testes de referência descritos em methodology.md.
- Não prometemos compatibilidade binária com toda versão antiga de glibc,
  Leap ou Ubuntu. Compile e empacote no sistema de destino.
