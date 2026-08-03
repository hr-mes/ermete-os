Name:           ermete-qa
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Quality Assurance Scripts

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge

BuildArch:      noarch

%description
Diagnostic and testing scripts for Ermete OS CI/CD.

%prep
# No prep

%build
# No build

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 %{_sourcedir}/test-nvidia-modules.sh %{buildroot}%{_bindir}/test-nvidia-modules.sh

%files
%{_bindir}/test-nvidia-modules.sh

%changelog
* Mon Aug 03 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release
