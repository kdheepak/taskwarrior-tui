
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'taskwarrior-tui' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'taskwarrior-tui'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'taskwarrior-tui' {
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Sets the data folder for taskwarrior-tui')
            [CompletionResult]::new('--data', 'data', [CompletionResultType]::ParameterName, 'Sets the data folder for taskwarrior-tui')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Sets the config folder for taskwarrior-tui (currently not used)')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Sets the config folder for taskwarrior-tui (currently not used)')
            [CompletionResult]::new('--taskdata', 'taskdata', [CompletionResultType]::ParameterName, 'Sets the .task folder using the TASKDATA environment variable for taskwarrior')
            [CompletionResult]::new('--taskrc', 'taskrc', [CompletionResultType]::ParameterName, 'Sets the .taskrc file using the TASKRC environment variable for taskwarrior')
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Sets default report')
            [CompletionResult]::new('--report', 'report', [CompletionResultType]::ParameterName, 'Sets default report')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
