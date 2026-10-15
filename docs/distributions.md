# Arch, Ubuntu, Fedora e openSUSE

O mesmo código Rust fornece CLI e TUI nas quatro famílias. Não depende de
GTK, Qt, X11, Wayland ou ncurses. A TUI usa Ratatui e Crossterm; requer um
terminal com suporte a sequências ANSI, UTF-8 e ao menos 64 × 22 células.
SSH também funciona se houver um terminal alocado (`ssh -t`).

A arquitetura principal investigada é Linux x86_64. A fórmula funciona em
outros alvos Rust, mas a carga de instruções não foi comparada em ARM.

## Pré-requisitos

Rust **1.88 ou mais recente**, Cargo, compilador/linker C e Git. O arquivo
`rust-toolchain.toml` seleciona stable quando o Cargo é gerido pelo rustup;
o `Cargo.lock` fixa as versões das dependências. Um Cargo nativo da distro
também pode compilar se sua versão de Rust atender ao mínimo. A validação
registrada usa Rust 1.99.0.

### Arch / EndeavourOS / Manjaro / Omarchy

```sh
sudo pacman -S --needed base-devel git curl rustup
rustup toolchain install stable --profile minimal
```

Use `rust` em vez de `rustup` se preferir uma toolchain nativa compatível.
Evite instalar simultaneamente pacotes conflitantes. O [pacote rustup do
Arch](https://archlinux.org/packages/extra/x86_64/rustup/) fornece os proxies
Cargo/Rust, mas a toolchain precisa estar disponível para o usuário.

### Ubuntu / Debian / derivados

```sh
sudo apt update
sudo apt install build-essential git curl ca-certificates
```

Instale Rust recente pelo procedimento universal abaixo. Não assuma que o
`rustc` do repositório de uma versão antiga do Ubuntu atende ao mínimo.
O build em Ubuntu 24.04 é o alvo fixado da matriz de CI.

### Fedora

```sh
sudo dnf install gcc make git curl rustup
rustup toolchain install stable --profile minimal
```

[Rustup é distribuído pelo Fedora](https://packages.fedoraproject.org/pkgs/rustup/rustup/).
Uma toolchain nativa recente, com `sudo dnf install rust cargo`, é outra
opção. Para gerar RPM, instale também `rpm-build`.

### openSUSE Tumbleweed / Leap

```sh
sudo zypper install gcc make git curl ca-certificates
```

Instale Rust recente pelo procedimento universal. Se o repositório de sua
versão disponibilizar `rustup`, ele também pode ser usado; não pressupomos
esse pacote em todas as versões de Leap. Para gerar RPM, instale `rpm-build`,
`tar` e `gzip`. A matriz de CI usa Tumbleweed.

### Rustup universal, sem executar diretamente um pipe remoto

Se Rust não estiver instalado ou for antigo:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup-init.sh
less rustup-init.sh
sh rustup-init.sh --profile minimal
. "$HOME/.cargo/env"
```

Revise o arquivo antes de executá-lo. O procedimento vem da [documentação
oficial do Rust](https://doc.rust-lang.org/book/ch01-01-installation.html).
Se já usa rustup, basta `rustup toolchain install stable --profile minimal`.
Nenhum dos scripts deste projeto instala Rust ou dependências de sistema.

## Compilar e instalar para o usuário

```sh
git clone https://github.com/kimlherme/cpuz-linux.git
cd cpuz-linux
cargo build --release --locked
./target/release/cpuz-linux --tui
./scripts/install.sh --prefix "$HOME/.local"
```

O instalador compila o programa, copia o binário, a página de manual e a
documentação para o prefixo escolhido. Não usa `sudo`, não altera o PATH,
não instala serviços nem muda afinidade/prioridade/políticas de energia.

```sh
"$HOME/.local/bin/cpuz-linux" --mode both --seconds 10 --threads auto
"$HOME/.local/bin/cpuz-linux" --tui
```

Se `~/.local/bin` já estiver no PATH, use `cpuz-linux`. Caso contrário,
use o caminho completo ou ajuste o PATH conforme seu shell.

Para instalar no sistema, compile como usuário e copie os arquivos para
um prefixo administrativo com as permissões apropriadas. Evite executar
Cargo como root. Builds feitos numa distro recente podem requerer glibc
mais recente do que a de outras distros; compile na distro de destino.

### Build sem rede

O primeiro `cargo fetch --locked` baixa as dependências. Depois:

```sh
cargo build --release --locked --offline
./scripts/install.sh --prefix "$HOME/.local" --offline
```

O modo offline exige toolchain e crates já presentes; não transforma um
ambiente vazio em um ambiente pronto para compilar.

### Remover

```sh
./scripts/install.sh --prefix "$HOME/.local" --uninstall
```

Remove somente o binário, manual e diretório de documentação do projeto
naquele prefixo. Não remove Rust, cache de Cargo nem pacotes da distro.

## Pacotes nativos

As receitas são ferramentas locais, não publicação no AUR, OBS, Fedora
Packages ou repositórios oficiais. Os scripts não instalam os pacotes
gerados. O diretório `dist/` fica fora do Git.

### Arch: PKGBUILD de desenvolvimento

```sh
cd packaging/arch
makepkg -si
```

Execute como usuário normal. O pacote `cpuz-linux-git` busca o código atual
do GitHub, compila com o lockfile, testa e instala o binário e a documentação.
`SKIP` no checksum é a convenção para uma fonte Git mutável, não validação
de um arquivo de release. Para reproduzir um experimento, registre o commit.

### Ubuntu: DEB

Na distro de destino, com `dpkg-deb` disponível:

```sh
./scripts/package.sh deb
dpkg-deb --info dist/cpuz-linux_0.2.0_amd64.deb
sudo apt install ./dist/cpuz-linux_0.2.0_amd64.deb
```

O pacote declara dependências de `libc6` e `libgcc-s1`. Ele é gerado a partir
do binário compilado no ambiente atual; não promete compatibilidade glibc
com versões mais antigas. O nome de arquitetura muda para `arm64` no ARM64.

### Fedora / openSUSE: RPM

```sh
# Fedora
sudo dnf install rpm-build
# openSUSE
sudo zypper install rpm-build tar gzip

./scripts/package.sh rpm
rpm -qp --info dist/*.rpm
```

Instale o RPM correspondente com `dnf install ./dist/arquivo.rpm` ou
`zypper install ./dist/arquivo.rpm`. O gerador RPM detecta requisitos do
ELF; gere o pacote em cada família de destino. A receita empacota um
binário compilado localmente, não é um pacote-fonte de revisão de distro.

## CI e validação

[A matriz GitHub Actions](../.github/workflows/linux.yml) usa Ubuntu 24.04,
Arch, Fedora 44 e openSUSE Tumbleweed. Compila, testa, verifica Clippy,
executa single/multi com JSON e testa instalação/remoção em um prefixo
temporário. Também gera DEB no Ubuntu e RPM nas famílias Fedora/openSUSE.

Arquivos de workflow descrevem verificações programadas; o resultado de
uma execução deve ser consultado no Actions. [validation.md](validation.md)
registra apenas verificações efetivamente executadas na sessão de entrega.
Não confunda um container com uma medição de desempenho em desktop nativo.

## Problemas comuns

| Sintoma | Ação |
| --- | --- |
| Cargo não encontrado | Carregue `~/.cargo/env`, reabra o shell ou use `~/.cargo/bin/cargo` |
| Rust antigo | Atualize a toolchain, mínimo 1.88 |
| Erro `cc` / linker | Instale GCC e os arquivos de desenvolvimento da libc |
| Tela cortada | Aumente o terminal; ajuste fonte e locale UTF-8 |
| `--tui` em pipe | Use `--json` ou abra um terminal real |
| Poucas CPUs disponíveis | Verifique cpuset, cgroups, container e `taskset` |
| RPM/DEB não abre em outra distro | Compile/empacote na distro de destino |
| Score diferente do CPU-Z | A equivalência não foi estabelecida; leia methodology.md |

Instalação e comparação de desempenho são temas distintos: funcionar nas
quatro distribuições não implica scores idênticos entre seus kernels,
compiladores, bibliotecas matemáticas ou políticas de energia.
