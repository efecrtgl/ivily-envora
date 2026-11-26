//! Yerleşik şablonlar. Şablonlar yalnızca **öneridir**; gerçek yapılandırma kullanıcı bir profil
//! veya proje oluşturup kaydettiğinde oluşur. Hiçbiri gizli değer içermez ve hiçbiri komut çalıştırmaz.

use super::model::Role;
use crate::env::Scope;
use crate::pathmgr::PathAction;

pub struct BVar {
    pub name: &'static str,
    pub value: &'static str,
    pub scope: Scope,
    pub role: Role,
    pub note: (&'static str, &'static str),
}

pub struct BPath {
    pub entry: &'static str,
    pub action: PathAction,
    pub scope: Scope,
}

pub struct Builtin {
    pub id: &'static str,
    pub category: &'static str,
    pub name: (&'static str, &'static str),
    pub description: (&'static str, &'static str),
    pub vars: &'static [BVar],
    pub paths: &'static [BPath],
}

use Role::*;
use Scope::*;

const fn v(
    name: &'static str,
    value: &'static str,
    scope: Scope,
    role: Role,
    tr: &'static str,
    en: &'static str,
) -> BVar {
    BVar {
        name,
        value,
        scope,
        role,
        note: (tr, en),
    }
}

const fn p(entry: &'static str, action: PathAction, scope: Scope) -> BPath {
    BPath {
        entry,
        action,
        scope,
    }
}

pub static BUILTINS: &[Builtin] = &[
    Builtin {
        id: "node",
        category: "Node.js",
        name: ("Node.js geliştirme", "Node.js development"),
        description: (
            "Node.js uygulamaları için yaygın ayarlar. NODE_ENV yalnızca başlatılan süreçte ayarlanır; kullanıcı genelinde ayarlamak npm kurulumlarını etkileyebilir.",
            "Common settings for Node.js apps. NODE_ENV is set only for the launched process; setting it user-wide can affect npm installs.",
        ),
        vars: &[
            v("NODE_ENV", "development", Process, Recommended, "Uygulamanın çalışma kipi.", "The application's runtime mode."),
            v("NODE_OPTIONS", "--max-old-space-size=4096", Process, Optional, "Büyük projelerde bellek sınırını artırır.", "Raises the memory limit for large projects."),
            v("NPM_CONFIG_FUND", "false", User, Optional, "npm'in bağış iletilerini gizler.", "Hides npm funding messages."),
        ],
        paths: &[p("%APPDATA%\\npm", PathAction::Append, User)],
    },
    Builtin {
        id: "python",
        category: "Python",
        name: ("Python geliştirme", "Python development"),
        description: ("Windows'ta Python için kodlama ve önbellek ayarları.", "Encoding and cache settings for Python on Windows."),
        vars: &[
            v("PYTHONUTF8", "1", User, Recommended, "Python'un dosya ve konsolda UTF-8 kullanmasını sağlar.", "Makes Python use UTF-8 for files and the console."),
            v("PYTHONDONTWRITEBYTECODE", "1", Process, Optional, "__pycache__ dosyalarının yazılmasını engeller.", "Prevents writing __pycache__ files."),
            v("PIP_DISABLE_PIP_VERSION_CHECK", "1", User, Optional, "pip'in sürüm denetimi uyarısını kapatır.", "Turns off pip's version check warning."),
        ],
        paths: &[],
    },
    Builtin {
        id: "rust",
        category: "Rust",
        name: ("Rust geliştirme", "Rust development"),
        description: ("Rust araç zinciri için PATH ve tanılama ayarları.", "PATH and diagnostics settings for the Rust toolchain."),
        vars: &[
            v("RUST_BACKTRACE", "1", Process, Recommended, "Panik durumunda yığın izini gösterir.", "Shows a backtrace on panic."),
            v("CARGO_TERM_COLOR", "always", User, Optional, "Cargo çıktısını her zaman renkli yapar.", "Always colors Cargo output."),
        ],
        paths: &[p("%USERPROFILE%\\.cargo\\bin", PathAction::Append, User)],
    },
    Builtin {
        id: "java",
        category: "Java",
        name: ("Java geliştirme", "Java development"),
        description: (
            "JDK konumu ve derleme araçları. JAVA_HOME değerini kendi JDK dizininizle doldurun.",
            "JDK location and build tools. Fill in JAVA_HOME with your own JDK directory.",
        ),
        vars: &[
            v("JAVA_HOME", "", User, Required, "JDK kök dizini, ör. C:\\Program Files\\Eclipse Adoptium\\jdk-21.", "JDK root directory, e.g. C:\\Program Files\\Eclipse Adoptium\\jdk-21."),
            v("MAVEN_OPTS", "-Xmx1g", User, Optional, "Maven için JVM bellek ayarı.", "JVM memory setting for Maven."),
        ],
        paths: &[p("%JAVA_HOME%\\bin", PathAction::Prepend, User)],
    },
    Builtin {
        id: "go",
        category: "Go",
        name: ("Go geliştirme", "Go development"),
        description: ("Go çalışma alanı ve modül vekil sunucusu.", "Go workspace and module proxy."),
        vars: &[
            v("GOPATH", "%USERPROFILE%\\go", User, Recommended, "Go modül önbelleği ve kurulan araçlar.", "Go module cache and installed tools."),
            v("GOPROXY", "https://proxy.golang.org,direct", User, Optional, "Modül vekil sunucusu.", "Module proxy."),
        ],
        paths: &[p("%USERPROFILE%\\go\\bin", PathAction::Append, User)],
    },
    Builtin {
        id: "php",
        category: "PHP",
        name: ("PHP geliştirme", "PHP development"),
        description: ("Composer ile kurulan genel araçlar için PATH.", "PATH for global tools installed with Composer."),
        vars: &[v("COMPOSER_HOME", "%APPDATA%\\Composer", User, Optional, "Composer'ın genel dizini.", "Composer's global directory.")],
        paths: &[p("%APPDATA%\\Composer\\vendor\\bin", PathAction::Append, User)],
    },
    Builtin {
        id: "dotnet",
        category: ".NET",
        name: (".NET geliştirme", ".NET development"),
        description: (".NET CLI ve ASP.NET Core için yaygın ayarlar.", "Common settings for the .NET CLI and ASP.NET Core."),
        vars: &[
            v("DOTNET_CLI_TELEMETRY_OPTOUT", "1", User, Recommended, ".NET CLI telemetrisini kapatır.", "Turns off .NET CLI telemetry."),
            v("DOTNET_NOLOGO", "1", User, Optional, "CLI karşılama iletilerini gizler.", "Hides CLI welcome messages."),
            v("ASPNETCORE_ENVIRONMENT", "Development", Process, Recommended, "ASP.NET Core ortam adı.", "ASP.NET Core environment name."),
        ],
        paths: &[p("%USERPROFILE%\\.dotnet\\tools", PathAction::Append, User)],
    },
    Builtin {
        id: "web",
        category: "Web",
        name: ("Web geliştirme", "Web development"),
        description: (
            "Yerel geliştirme sunucuları için süreç değişkenleri. Bunlar yalnızca proje başlatılırken uygulanır.",
            "Process variables for local development servers. These are applied only when a project is launched.",
        ),
        vars: &[
            v("NODE_ENV", "development", Process, Recommended, "Uygulamanın çalışma kipi.", "The application's runtime mode."),
            v("PORT", "3000", Process, Optional, "Geliştirme sunucusunun dinleyeceği bağlantı noktası.", "Port the development server listens on."),
            v("HOST", "localhost", Process, Optional, "Geliştirme sunucusunun adresi.", "Address of the development server."),
            v("BROWSER", "none", Process, Optional, "Bazı araçların tarayıcıyı otomatik açmasını engeller.", "Prevents some tools from opening the browser automatically."),
        ],
        paths: &[],
    },
    Builtin {
        id: "docker",
        category: "Docker",
        name: ("Docker", "Docker"),
        description: ("Docker derleme ayarları.", "Docker build settings."),
        vars: &[
            v("DOCKER_BUILDKIT", "1", User, Recommended, "BuildKit derleyicisini kullanır.", "Uses the BuildKit builder."),
            v("COMPOSE_PROJECT_NAME", "", Process, Optional, "Compose proje adı (boş bırakılırsa dizin adı kullanılır).", "Compose project name (directory name is used if empty)."),
        ],
        paths: &[],
    },
    Builtin {
        id: "database",
        category: "Veritabanı",
        name: ("Veritabanı geliştirme", "Database development"),
        description: (
            "Yerel veritabanı bağlantısı. Şablon parola içermez; parola içeren bağlantı dizesini projeye özgü (şifreli saklanan) değişken olarak girin.",
            "Local database connection. The template contains no password; enter a connection string with a password as a project-specific (encrypted) variable.",
        ),
        vars: &[
            v("DATABASE_URL", "", Process, Required, "Bağlantı dizesi, ör. postgres://kullanici@localhost:5432/veritabani", "Connection string, e.g. postgres://user@localhost:5432/database"),
            v("PGHOST", "localhost", Process, Optional, "PostgreSQL sunucusu.", "PostgreSQL host."),
            v("PGPORT", "5432", Process, Optional, "PostgreSQL bağlantı noktası.", "PostgreSQL port."),
        ],
        paths: &[],
    },
    Builtin {
        id: "custom",
        category: "",
        name: ("Boş şablon", "Blank template"),
        description: ("Kendi değişkenlerinizi eklemek için boş başlangıç.", "An empty starting point for your own variables."),
        vars: &[],
        paths: &[],
    },
];
