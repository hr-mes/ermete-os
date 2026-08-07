Name:           ermete-antigravity
Version:        2.0.0
Release:        1%{?dist}
Summary:        Ermete OS Antigravity Swarm CLI & Orchestration Engine

License:        Proprietary
URL:            https://github.com/hr-mes/ermete-os

Requires:       python3, python3-pip, podman

%description
Packages the Antigravity Swarm Agent CLI and Ring-0 orchestration scripts as native RPMs.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Antigravity Swarm Engine'" > %{buildroot}/usr/bin/agy
chmod +x %{buildroot}/usr/bin/agy

%files
/usr/bin/agy

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 2.0.0-1
- Initial Antigravity Swarm CLI spec.
