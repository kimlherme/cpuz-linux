Name:           cpuz-linux
Version:        0.2.0
Release:        1%{?dist}
Summary:        Experimental CPU benchmark with a terminal interface
License:        GPL-2.0-only
URL:            https://github.com/kimlherme/cpuz-linux
Source0:        cpuz-linux
Source1:        README.md
Source2:        LICENSE
Source3:        cpuz-linux.1
Source4:        docs.tar.gz

%description
Single and multi thread procedural noise benchmark written in Rust.
Includes CPU identification and an interactive terminal interface.
Scores are experimental and have not been validated as CPU-Z equivalents.
This recipe packages the binary built locally by scripts/package.sh.

%prep

%build

%install
install -Dpm755 %{SOURCE0} %{buildroot}%{_bindir}/cpuz-linux
install -Dpm644 %{SOURCE3} %{buildroot}%{_mandir}/man1/cpuz-linux.1
install -Dpm644 %{SOURCE1} %{buildroot}%{_docdir}/%{name}/README.md
install -Dpm644 %{SOURCE2} %{buildroot}%{_licensedir}/%{name}/LICENSE
mkdir -p %{buildroot}%{_docdir}/%{name}
tar -xzf %{SOURCE4} -C %{buildroot}%{_docdir}/%{name}

%files
%{_bindir}/cpuz-linux
%{_mandir}/man1/cpuz-linux.1*
%{_docdir}/%{name}/
%license %{_licensedir}/%{name}/LICENSE
