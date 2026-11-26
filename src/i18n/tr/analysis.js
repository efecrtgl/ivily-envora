// Ortam analizi (Türkçe). Kural anahtarlarında nokta yerine alt çizgi kullanılır.
export default {
  analysis: {
    title: 'Ortam analizi',
    subtitle: 'Ortamınızdaki olası sorunları bulur ve açıklar. Hiçbir şey otomatik düzeltilmez.',
    run: 'Analiz et',
    running: 'Analiz ediliyor…',
    rulesRun: 'Çalışan kural',
    rulesTitle: 'Kurallar',
    rulesDesc: 'Kuralları açıp kapatabilir ve önem derecelerini değiştirebilirsiniz.',
    severity: 'Önem',
    count: { error: 'Hata', warning: 'Uyarı', info: 'Bilgi' },
    cleanTitle: 'Sorun bulunamadı',
    cleanBody: 'Etkin kuralların hiçbiri bulgu üretmedi.',
    goTo: 'İlgili sayfa',
    note: 'Analiz yalnızca siz başlattığınızda veya panoyu açtığınızda çalışır; sürekli arka plan taraması yapılmaz. Bulgular hiçbir zaman değişken değeri içermez.',
    category: {
      path: 'PATH',
      variables: 'Değişkenler',
      tools: 'Araçlar',
      security: 'Güvenlik',
      system: 'Sistem',
      process: 'İşlem',
      profiles: 'Profiller',
    },
    rule: {
      path_missing: {
        title: 'PATH dizini bulunamadı',
        body: 'Bu girdi var olmayan bir dizini veya bir dosyayı gösteriyor ({index}. sıra). Program kaldırılmış olabilir; girdi komut aramayı yavaşlatır ve kafa karıştırır. PATH yöneticisinden kaldırmayı değerlendirin.',
      },
      path_duplicate: {
        title: 'Yinelenen PATH girdisi',
        body: 'Aynı dizin bu kapsamda daha önce geçiyor ({index}. sıra). Sonraki kopyanın etkisi yoktur; kaldırmak PATH\'i kısaltır.',
      },
      path_crossScopeDuplicate: {
        title: 'Hem kullanıcı hem sistem PATH\'inde',
        body: 'Bu dizin her iki kapsamda da var ve birleşik PATH\'te iki kez geçiyor. Genellikle birinde tutmak yeterlidir.',
      },
      path_relative: {
        title: 'Göreli PATH girdisi',
        body: 'Göreli girdiler, komutların çalışma dizinine göre aranmasına yol açar; başka bir dizindeki aynı adlı programın çalışmasına neden olabilir.',
      },
      path_empty: {
        title: 'Boş PATH girdisi',
        body: 'Arka arkaya iki noktalı virgül boş bir girdi oluşturuyor ({index}. sıra). Zararsızdır ama gereksizdir.',
      },
      path_unresolved: {
        title: 'Çözülemeyen PATH başvurusu',
        body: 'Girdi tanımlı olmayan bir %AD% başvurusu içeriyor ({index}. sıra); Windows bu girdiyi kullanamaz.',
      },
      path_length: {
        title: 'PATH çok uzun',
        body: 'PATH {length} karakter; {limit} karakter sınırını aşıyor. Bazı araçlar ve kurulum programları PATH\'i kesebilir. Yinelenen ve bulunamayan girdileri kaldırmak yardımcı olur.',
      },
      var_name: {
        title: 'Sorunlu değişken adı',
        body: 'Ad boşluk içeriyor veya rakamla başlıyor; bazı kabuklar bu değişkene erişemez.',
      },
      var_whitespace: {
        title: 'Değerde baştaki/sondaki boşluk',
        body: 'Değerin başında veya sonunda boşluk var. Yol ve sürüm numaralarında bu genellikle istenmeyen bir durumdur.',
      },
      var_unexpanded: {
        title: '%AD% başvurusu genişletilmiyor',
        body: 'Değer %AD% başvurusu içeriyor ancak türü "Metin" (REG_SZ). Windows bu başvuruyu genişletmez; programlar değeri olduğu gibi görür. Türü "Genişletilebilir" yapmayı değerlendirin.',
      },
      var_undefinedReference: {
        title: 'Tanımsız değişkene başvuru',
        body: 'Değer %{reference}% değişkenine başvuruyor ancak bu değişken tanımlı değil; o kısım genişletilmez.',
      },
      var_empty: {
        title: 'Boş değer',
        body: 'Kalıcı bir değişkenin değeri boş. Bazı araçlar boş değeri "tanımsız" ile aynı saymaz.',
      },
      var_shadowed: {
        title: 'Kullanıcı değeri sistem değerini gölgeliyor',
        body: 'Aynı ad sistem kapsamında farklı bir değerle de var. Yeni işlemler kullanıcı değerini görür; sistem değeri bu kullanıcı için etkisizdir.',
      },
      var_homeMissing: {
        title: 'Araç dizini bulunamadı',
        body: 'Bu *_HOME/*_ROOT/*_DIR değişkeni var olmayan bir dizini gösteriyor. Araç kaldırılmış veya taşınmış olabilir.',
      },
      tools_homeNotInPath: {
        title: 'Araç dizini PATH\'te değil',
        body: '{dir} dizini var ancak PATH\'te değil; bu aracın komutları başka bir sürümden çalışıyor veya hiç bulunamıyor olabilir.',
      },
      tools_storeAlias: {
        title: 'Store takma adı gerçek kurulumu gölgeliyor',
        body: 'PATH\'te Microsoft Store uygulama yürütme takma adı gerçek kurulumdan önce geliyor. Komut, kurulu sürüm yerine Store\'u açabilir. Ayarlar → Uygulamalar → Gelişmiş uygulama ayarları → Uygulama yürütme takma adları bölümünden kapatılabilir.',
      },
      system_tempMissing: {
        title: 'Geçici dizin bulunamadı',
        body: 'TEMP/TMP var olmayan bir dizini gösteriyor. Birçok program ve kurulum aracı çalışamaz.',
      },
      system_pathext: {
        title: 'PATHEXT eksik',
        body: 'PATHEXT şu uzantıları içermiyor: {missing}. Bu uzantılı programlar adlarıyla çalıştırılamaz.',
      },
      security_machineSecret: {
        title: 'Sistem kapsamında gizli değer',
        body: 'Hassas görünen bir değer sistem kapsamında; bu bilgisayardaki tüm kullanıcılar ve hizmetler okuyabilir. Kullanıcı kapsamına veya yalnızca gerektiğinde işlem kapsamına (proje başlatma) taşımayı değerlendirin.',
      },
      security_persistentSecret: {
        title: 'Kalıcı ortamda gizli değer',
        body: 'Kalıcı ortam değişkenleri kayıt defterinde düz metin olarak durur ve başlattığınız her program tarafından görülür. Yalnızca belirli projeler ihtiyaç duyuyorsa proje değişkeni olarak (DPAPI ile şifreli) tutmak daha güvenlidir.',
      },
      process_stale: {
        title: 'Envora\'nın işlem ortamı eski',
        body: 'Envora başlatıldıktan sonra {count} kalıcı değişken değişmiş. Açık programlar eski değerleri görmeye devam eder; yeni değerler için yeniden başlatılmaları gerekir.',
      },
      profile_conflict: {
        title: 'Etkin profiller çakışıyor',
        body: '"{a}" ve "{b}" profilleri bu değişkeni aynı kapsamda farklı değerlerle ayarlıyor; en son uygulanan geçerlidir.',
      },
      profile_drift: {
        title: 'Profil değeri değişmiş',
        body: '"{profile}" profili etkin ancak bu değişkenin ortamdaki değeri profildekinden farklı (elle değiştirilmiş veya kaldırılmış olabilir).',
      },
    },
  },
};
