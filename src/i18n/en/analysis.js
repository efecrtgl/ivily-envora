// Environment analysis (English). Rule keys use underscores instead of dots.
export default {
  analysis: {
    title: 'Environment analysis',
    subtitle: 'Finds and explains possible problems in your environment. Nothing is fixed automatically.',
    run: 'Analyze',
    running: 'Analyzing…',
    rulesRun: 'Rules run',
    rulesTitle: 'Rules',
    rulesDesc: 'You can turn rules on or off and change their severity.',
    severity: 'Severity',
    count: { error: 'Errors', warning: 'Warnings', info: 'Info' },
    cleanTitle: 'No problems found',
    cleanBody: 'None of the enabled rules produced a finding.',
    goTo: 'Related page',
    note: 'Analysis runs only when you start it or open the dashboard; there is no continuous background scanning. Findings never contain variable values.',
    category: {
      path: 'PATH',
      variables: 'Variables',
      tools: 'Tools',
      security: 'Security',
      system: 'System',
      process: 'Process',
      profiles: 'Profiles',
    },
    rule: {
      path_missing: {
        title: 'PATH directory not found',
        body: 'This entry points to a directory that does not exist, or to a file (position {index}). The program may have been uninstalled; the entry slows command lookup and can be confusing. Consider removing it in the PATH manager.',
      },
      path_duplicate: {
        title: 'Duplicate PATH entry',
        body: 'The same directory appears earlier in this scope (position {index}). The later copy has no effect; removing it shortens PATH.',
      },
      path_crossScopeDuplicate: {
        title: 'In both user and system PATH',
        body: 'This directory exists in both scopes and appears twice in the combined PATH. Keeping it in one of them is usually enough.',
      },
      path_relative: {
        title: 'Relative PATH entry',
        body: 'Relative entries make commands resolve against the working directory, which can cause a same-named program in another directory to run.',
      },
      path_empty: {
        title: 'Empty PATH entry',
        body: 'Consecutive semicolons create an empty entry (position {index}). Harmless but unnecessary.',
      },
      path_unresolved: {
        title: 'Unresolved PATH reference',
        body: 'The entry contains a %NAME% reference that is not defined (position {index}); Windows cannot use this entry.',
      },
      path_length: {
        title: 'PATH is too long',
        body: 'PATH is {length} characters, over the {limit}-character limit. Some tools and installers may truncate it. Removing duplicate and missing entries helps.',
      },
      var_name: {
        title: 'Problematic variable name',
        body: 'The name contains whitespace or starts with a digit; some shells cannot access this variable.',
      },
      var_whitespace: {
        title: 'Leading/trailing whitespace in value',
        body: 'The value has leading or trailing whitespace. For paths and version numbers this is usually unintended.',
      },
      var_unexpanded: {
        title: '%NAME% reference is not expanded',
        body: 'The value contains a %NAME% reference but its type is "String" (REG_SZ). Windows does not expand it; programs see the value literally. Consider making the type "Expandable".',
      },
      var_undefinedReference: {
        title: 'Reference to an undefined variable',
        body: 'The value references %{reference}%, which is not defined; that part is not expanded.',
      },
      var_empty: {
        title: 'Empty value',
        body: 'A persistent variable has an empty value. Some tools do not treat an empty value the same as "not defined".',
      },
      var_shadowed: {
        title: 'User value shadows system value',
        body: 'The same name also exists in the system scope with a different value. New processes see the user value; the system value has no effect for this user.',
      },
      var_homeMissing: {
        title: 'Tool directory not found',
        body: 'This *_HOME/*_ROOT/*_DIR variable points to a directory that does not exist. The tool may have been uninstalled or moved.',
      },
      tools_homeNotInPath: {
        title: 'Tool directory not on PATH',
        body: 'The directory {dir} exists but is not on PATH; commands for this tool may be running from another version or not found at all.',
      },
      tools_storeAlias: {
        title: 'Store alias shadows the real installation',
        body: "A Microsoft Store app execution alias comes before the real installation on PATH. The command may open the Store instead of the installed version. It can be turned off in Settings → Apps → Advanced app settings → App execution aliases.",
      },
      system_tempMissing: {
        title: 'Temporary directory not found',
        body: 'TEMP/TMP points to a directory that does not exist. Many programs and installers cannot work.',
      },
      system_pathext: {
        title: 'PATHEXT is incomplete',
        body: 'PATHEXT does not contain: {missing}. Programs with these extensions cannot be run by name.',
      },
      security_machineSecret: {
        title: 'Secret value in system scope',
        body: 'A value that looks sensitive is in the system scope, readable by every user and service on this computer. Consider moving it to the user scope, or to the process scope only when needed (project launch).',
      },
      security_persistentSecret: {
        title: 'Secret value in the persistent environment',
        body: 'Persistent environment variables are stored in plain text in the registry and are visible to every program you start. If only specific projects need it, keeping it as a project variable (encrypted with DPAPI) is safer.',
      },
      process_stale: {
        title: "Envora's process environment is outdated",
        body: '{count} persistent variables changed after Envora started. Open programs keep seeing the old values; they need to be restarted to see the new ones.',
      },
      profile_conflict: {
        title: 'Active profiles conflict',
        body: 'Profiles "{a}" and "{b}" set this variable in the same scope with different values; the last one applied wins.',
      },
      profile_drift: {
        title: 'Profile value changed',
        body: 'Profile "{profile}" is active, but this variable\'s value in the environment differs from the profile (it may have been changed or removed manually).',
      },
    },
  },
};
