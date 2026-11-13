#include <QGuiApplication>
#include <QProcessEnvironment>
#include <QQmlApplicationEngine>

namespace {

const QString prefix = QStringLiteral("BENCH_");

// The BENCH_* variables the harness sets, keyed like the web query string: "row_height", ...
QVariantMap scenarioConfig()
{
    const auto environment = QProcessEnvironment::systemEnvironment();
    QVariantMap config;
    for (const QString &name : environment.keys()) {
        if (name.startsWith(prefix))
            config.insert(name.mid(prefix.size()).toLower(), environment.value(name));
    }
    return config;
}

}

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;
    engine.setInitialProperties({{QStringLiteral("config"), scenarioConfig()}});
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app,
        [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.loadFromModule("Bench", "Main");
    return app.exec();
}
