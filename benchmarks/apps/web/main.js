const root = document.createElement('div');
document.body.append(root);
const scenarios = {
  hello: mountHello,
  list: mountList,
  animation: mountAnimation,
  settings: mountSettings,
};
scenarios[benchConfig.scenario](root, benchConfig);
