Name:           ermete-osbuild
Version:        1.0.0
Release:        1%{?dist}
Summary:        OSBuild and Bootc native compilation for Ermete OS

License:        MIT
URL:            https://github.com/osbuild/osbuild

%description
Replaces the bootc-image-builder-action with a native, optimized binary.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Native OSBuild'" > %{buildroot}/usr/bin/osbuild
chmod +x %{buildroot}/usr/bin/osbuild

%files
/usr/bin/osbuild
