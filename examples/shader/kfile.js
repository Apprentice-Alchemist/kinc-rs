let project = new Project("shader");

project.addFiles("Shaders/**");

project.setDebugDir('Deployment');

project.flatten();

resolve(project);