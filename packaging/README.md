# IPP Sharing @VERSION@ (Windows @ARCHITECTURE@)

This is a portable build of IPP Sharing. Extract the 7z file to a folder and
keep all files together.

## First run

1. Install Bonjour for service discovery. You can use WinGet:

   ```powershell
   winget install --id Apple.Bonjour -e
   ```

   Or, when using this portable artifact, install the included `Bonjour.msi`
   manually as an administrator. IPP Sharing does not install or start Bonjour
   for you. Restarting Windows is normally not needed.
2. Copy `config.example.yaml` to `config.yaml` and edit the printer entries.
   `target` must match the name of a printer already installed in Windows.
3. Start `ipp-sharing-gui.exe`, or run `ipp-sharing.exe` from a terminal to
   see command-line logs.

The default configuration listens on TCP port 631. Allow TCP 631 and UDP 5353
through Windows Firewall when clients cannot discover or connect to the
shared printer.

This package is @ARCHITECTURE@-only. See `LICENSE.md` for the application
license.
