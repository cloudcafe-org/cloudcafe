use color_eyre::Result;
#[derive(Copy, Clone, Debug)]
pub enum ScriptType {
    EnableDisplayDriver,
    DisableDisplayDriver,
}
impl ScriptType {
    fn to_script(self) -> String {
        String::from(match self {
            ScriptType::EnableDisplayDriver => {
r#"$ConfirmPreference = 'None'
Get-PnpDevice -FriendlyName "IddSampleDriver Device" | Enable-PnpDevice
"#
            }
            ScriptType::DisableDisplayDriver => {
r#"$ConfirmPreference = 'None'
Get-PnpDevice -FriendlyName "IddSampleDriver Device" | Disable-PnpDevice
"#
            }
        })
    }
    pub fn run(self) -> Result<()> {
        let powershell_runner = powershell_script::PsScriptBuilder::new().hidden(false).print_commands(true).build();
        powershell_runner.run(self.to_script().as_str())?;
        Ok(())
    }
}