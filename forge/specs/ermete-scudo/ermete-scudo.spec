Name:           ermete-scudo
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Scudo Hardened Allocator Configuration

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge

BuildRequires:  systemd-rpm-macros
Requires:       compiler-rt

%description
Sets up Scudo standalone allocator via LD_PRELOAD globally for Ermete OS.

%prep
# No prep

%build
# No build

%install
mkdir -p %{buildroot}%{_sysconfdir}
mkdir -p %{buildroot}%{_prefix}/lib/environment.d
mkdir -p %{buildroot}%{_unitdir}/greetd.service.d
mkdir -p %{buildroot}%{_unitdir}/ermete-llm.service.d

# LD_PRELOAD
echo "/usr/lib64/libscudo.so" > %{buildroot}%{_sysconfdir}/ld.so.preload

# Scudo Options
cat <<EOF > %{buildroot}%{_prefix}/lib/environment.d/10-scudo.conf
SCUDO_OPTIONS="ZeroContents=1:PatternFillRet=1:DeallocationTypeMismatch=1:DeleteSizeMismatch=1"
EOF

# Greetd override
cat <<EOF > %{buildroot}%{_unitdir}/greetd.service.d/override.conf
[Service]
Environment="LD_PRELOAD="
EOF

# Ermete LLM override
cat <<EOF > %{buildroot}%{_unitdir}/ermete-llm.service.d/override.conf
[Service]
Environment="LD_PRELOAD="
EOF

%post
# Create symlink dynamically based on compiler-rt installed
if [ -d /usr/lib64/clang ]; then
  SCUDO_LIB=$(find /usr/lib64/clang -name "libclang_rt.scudo_standalone.so" | head -n 1)
  if [ -n "$SCUDO_LIB" ]; then
    ln -sf "$SCUDO_LIB" /usr/lib64/libscudo.so
  fi
fi

%files
%config(noreplace) %{_sysconfdir}/ld.so.preload
%{_prefix}/lib/environment.d/10-scudo.conf
%{_unitdir}/greetd.service.d/override.conf
%{_unitdir}/ermete-llm.service.d/override.conf

%changelog
* Mon Aug 03 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release
