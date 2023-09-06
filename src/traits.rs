use task_hookrs::task::Task;

pub trait TaskwarriorTuiTask {
  fn add_tag(&mut self, tag: String);

  fn remove_tag(&mut self, tag: &str);
}

impl TaskwarriorTuiTask for Task {
  fn add_tag(&mut self, tag: String) {
    match self.tags_mut() {
      Some(t) => t.push(tag),
      None => self.set_tags(Some(vec![tag])),
    }
  }

  fn remove_tag(&mut self, tag: &str) {
    if let Some(t) = self.tags_mut() {
      if let Some(index) = t.iter().position(|x| *x == tag) {
        t.remove(index);
      }
    }
  }
}
